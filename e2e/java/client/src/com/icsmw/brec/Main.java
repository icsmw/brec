package com.icsmw.brec;

import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.WebSocket;
import java.nio.ByteBuffer;
import java.io.ByteArrayOutputStream;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionStage;

public final class Main {
    private static final class EchoListener implements WebSocket.Listener {
        private final int expectedCount;
        private final CompletableFuture<Integer> completion = new CompletableFuture<>();

        private int packetCount = 0;
        private long recvBytes = 0;
        private long sentBytes = 0;
        private final ByteArrayOutputStream frameBuffer = new ByteArrayOutputStream();

        EchoListener(int expectedCount) {
            this.expectedCount = expectedCount;
        }

        CompletableFuture<Integer> completion() {
            return completion;
        }

        @Override
        public void onOpen(WebSocket webSocket) {
            System.out.println("[client] connected");
            webSocket.request(1);
        }

        @Override
        public CompletionStage<?> onBinary(WebSocket webSocket, ByteBuffer data, boolean last) {
            try {
                byte[] chunk = new byte[data.remaining()];
                data.get(chunk);
                frameBuffer.write(chunk);
                if (!last) {
                    webSocket.request(1);
                    return null;
                }

                byte[] in = frameBuffer.toByteArray();
                frameBuffer.reset();
                recvBytes += in.length;

                Object packet = ClientBindings.decodePacket(in);
                byte[] out = ClientBindings.encodePacket(packet);
                Object packetAfter = ClientBindings.decodePacket(out);

                if (!Objects.equals(packet, packetAfter)) {
                    throw new RuntimeException("client reencode mismatch");
                }

                sentBytes += out.length;
                packetCount += 1;

                webSocket.sendBinary(ByteBuffer.wrap(out), true);

                if (packetCount % 10 == 0 || packetCount == expectedCount) {
                    System.out.printf("[client] progress: %d/%d%n", packetCount, expectedCount);
                }
            } catch (Throwable err) {
                completion.completeExceptionally(err);
                webSocket.abort();
                return null;
            }

            webSocket.request(1);
            return null;
        }

        @Override
        public CompletionStage<?> onClose(WebSocket webSocket, int statusCode, String reason) {
            if (packetCount != expectedCount) {
                completion.completeExceptionally(
                    new RuntimeException(
                        String.format(
                            "CLIENT_SUMMARY packets=%d sent_bytes=%d recv_bytes=%d verified=false expected=%d",
                            packetCount,
                            sentBytes,
                            recvBytes,
                            expectedCount)));
            } else {
                System.out.printf(
                    "CLIENT_SUMMARY packets=%d sent_bytes=%d recv_bytes=%d verified=true%n",
                    packetCount,
                    sentBytes,
                    recvBytes);
                completion.complete(0);
            }
            return null;
        }

        @Override
        public void onError(WebSocket webSocket, Throwable error) {
            completion.completeExceptionally(error);
        }
    }

    public static void main(String[] args) {
        String wsAddr = System.getenv().getOrDefault("TEST_WS_ADDR", "host.docker.internal:19001");
        int expectedCount = Integer.parseInt(System.getenv().getOrDefault("TEST_PACKAGE_COUNT", "1000"));
        String wsUrl = "ws://" + wsAddr;

        EchoListener listener = new EchoListener(expectedCount);

        try {
            HttpClient.newHttpClient()
                .newWebSocketBuilder()
                .buildAsync(URI.create(wsUrl), listener)
                .join();

            listener.completion().join();
        } catch (Throwable err) {
            String msg = err.getMessage() == null ? err.toString() : err.getMessage();
            System.err.println("[client] error: " + msg);
            System.exit(1);
        }
    }
}
