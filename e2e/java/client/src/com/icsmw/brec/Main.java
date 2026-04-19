package com.icsmw.brec;

import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.WebSocket;
import java.nio.ByteBuffer;
import java.io.ByteArrayOutputStream;
import java.math.BigInteger;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionStage;

public final class Main {
    private static BigInteger f64Bits(double value) {
        long bits = Double.doubleToRawLongBits(value);
        return new BigInteger(Long.toUnsignedString(bits));
    }

    private static Long f32Bits(float value) {
        return Long.valueOf(Float.floatToRawIntBits(value));
    }

    @SuppressWarnings("unchecked")
    private static Object deepCopy(Object value) {
        if (value instanceof Map<?, ?> map) {
            HashMap<String, Object> copy = new HashMap<>();
            for (Map.Entry<?, ?> e : map.entrySet()) {
                copy.put(String.valueOf(e.getKey()), deepCopy(e.getValue()));
            }
            return copy;
        }
        if (value instanceof List<?> list) {
            ArrayList<Object> copy = new ArrayList<>(list.size());
            for (Object item : list) {
                copy.add(deepCopy(item));
            }
            return copy;
        }
        return value;
    }

    private static void expectEncodeFailure(Object packetLike, String label, String... mustContain) {
        String message = null;
        try {
            ClientBindings.encodePacket(packetLike);
        } catch (RuntimeException expected) {
            message = expected.getMessage() == null ? expected.toString() : expected.getMessage();
        } catch (Throwable unexpected) {
            throw new RuntimeException("unexpected throwable type for " + label, unexpected);
        }
        if (message == null) {
            throw new RuntimeException("expected encode failure: " + label);
        }
        if (!message.contains("encode packet failed")) {
            throw new RuntimeException("unexpected encode error prefix for " + label + ": " + message);
        }
        for (String part : mustContain) {
            if (!message.contains(part)) {
                throw new RuntimeException(
                    "encode failure message for " + label + " must contain '" + part + "', got: " + message);
            }
        }
    }

    private static HashMap<String, Object> buildValidPacketObject() {
        HashMap<String, Object> blockFields = new HashMap<>();
        blockFields.put("field_u8", Long.valueOf(1));
        blockFields.put("field_u16", Long.valueOf(2));
        blockFields.put("field_u32", Long.valueOf(3));
        blockFields.put("field_u64", new BigInteger("4"));
        blockFields.put("field_u128", new BigInteger("5"));
        blockFields.put("field_i8", Long.valueOf(-1));
        blockFields.put("field_i16", Long.valueOf(-2));
        blockFields.put("field_i32", Long.valueOf(-3));
        blockFields.put("field_i64", new BigInteger("-4"));
        blockFields.put("field_i128", new BigInteger("-5"));
        blockFields.put("field_f32", f32Bits(1.25f));
        blockFields.put("field_f64", f64Bits(2.5));
        blockFields.put("field_bool", Boolean.TRUE);

        HashMap<String, Object> blockVariant = new HashMap<>();
        blockVariant.put("BlockCombination", blockFields);
        ArrayList<Object> blocks = new ArrayList<>();
        blocks.add(blockVariant);

        HashMap<String, Object> payloadFields = new HashMap<>();
        payloadFields.put("field_u8", Long.valueOf(10));
        payloadFields.put("field_u16", Long.valueOf(11));
        payloadFields.put("field_u32", Long.valueOf(12));
        payloadFields.put("field_u64", new BigInteger("13"));
        payloadFields.put("field_u128", new BigInteger("14"));
        payloadFields.put("field_i8", Long.valueOf(-10));
        payloadFields.put("field_i16", Long.valueOf(-11));
        payloadFields.put("field_i32", Long.valueOf(-12));
        payloadFields.put("field_i64", new BigInteger("-13"));
        payloadFields.put("field_i128", new BigInteger("-14"));
        payloadFields.put("field_f32", f32Bits(3.5f));
        payloadFields.put("field_f64", f64Bits(4.5));
        payloadFields.put("field_bool", Boolean.FALSE);
        payloadFields.put("field_str", "coverage-java");

        payloadFields.put("vec_u8", new ArrayList<>(List.of(Long.valueOf(1), Long.valueOf(2))));
        payloadFields.put("vec_u16", new ArrayList<>(List.of(Long.valueOf(3), Long.valueOf(4))));
        payloadFields.put("vec_u32", new ArrayList<>(List.of(Long.valueOf(5), Long.valueOf(6))));
        payloadFields.put("vec_u64", new ArrayList<>(List.of(new BigInteger("7"), new BigInteger("8"))));
        payloadFields.put(
            "vec_u128",
            new ArrayList<>(List.of(new BigInteger("9"), new BigInteger("10"))));
        payloadFields.put("vec_i8", new ArrayList<>(List.of(Long.valueOf(-1), Long.valueOf(-2))));
        payloadFields.put("vec_i16", new ArrayList<>(List.of(Long.valueOf(-3), Long.valueOf(-4))));
        payloadFields.put("vec_i32", new ArrayList<>(List.of(Long.valueOf(-5), Long.valueOf(-6))));
        payloadFields.put("vec_i64", new ArrayList<>(List.of(new BigInteger("-7"), new BigInteger("-8"))));
        payloadFields.put(
            "vec_i128",
            new ArrayList<>(List.of(new BigInteger("-9"), new BigInteger("-10"))));
        payloadFields.put("vec_str", new ArrayList<>(List.of("a", "b")));

        HashMap<String, Object> payloadVariant = new HashMap<>();
        payloadVariant.put("PayloadA", payloadFields);

        HashMap<String, Object> packet = new HashMap<>();
        packet.put("blocks", blocks);
        packet.put("payload", payloadVariant);
        return packet;
    }

    @SuppressWarnings("unchecked")
    private static void runCoverageProbes() {
        HashMap<String, Object> valid = buildValidPacketObject();
        byte[] encoded = ClientBindings.encodePacket(valid);
        Object decoded = ClientBindings.decodePacket(encoded);
        if (!(decoded instanceof Map)) {
            throw new RuntimeException("coverage probe: decode did not return packet map");
        }
        byte[] encodedAgain = ClientBindings.encodePacket(decoded);
        Object decodedAgain = ClientBindings.decodePacket(encodedAgain);
        if (!Objects.equals(decoded, decodedAgain)) {
            throw new RuntimeException("coverage probe: decode/encode stability mismatch");
        }

        expectEncodeFailure(null, "null packet object", "null packet object");

        HashMap<String, Object> missingBlocks = (HashMap<String, Object>) deepCopy(valid);
        missingBlocks.remove("blocks");
        expectEncodeFailure(missingBlocks, "missing blocks", "Missing field: blocks");

        HashMap<String, Object> badBlocksType = (HashMap<String, Object>) deepCopy(valid);
        HashMap<String, Object> malformedBlock = new HashMap<>();
        malformedBlock.put("A", new HashMap<String, Object>());
        malformedBlock.put("B", new HashMap<String, Object>());
        badBlocksType.put("blocks", new ArrayList<>(List.of(malformedBlock)));
        expectEncodeFailure(
            badBlocksType,
            "invalid blocks type",
            "Invalid field value for blocks",
            "Invalid aggregator object shape");

        HashMap<String, Object> badBlockVariant = (HashMap<String, Object>) deepCopy(valid);
        ArrayList<Object> badBlockList = new ArrayList<>();
        HashMap<String, Object> unknownBlock = new HashMap<>();
        unknownBlock.put("UnknownBlock", new HashMap<String, Object>());
        badBlockList.add(unknownBlock);
        badBlockVariant.put("blocks", badBlockList);
        expectEncodeFailure(
            badBlockVariant,
            "invalid block variant",
            "Invalid field value for blocks",
            "Invalid aggregator object shape");

        HashMap<String, Object> payloadMissingIsOk = (HashMap<String, Object>) deepCopy(valid);
        payloadMissingIsOk.remove("payload");
        byte[] noPayload = ClientBindings.encodePacket(payloadMissingIsOk);
        ClientBindings.decodePacket(noPayload);

        HashMap<String, Object> nullBool = (HashMap<String, Object>) deepCopy(valid);
        Map<String, Object> blockVariant = (Map<String, Object>) ((List<Object>) nullBool.get("blocks")).get(0);
        Map<String, Object> blockFields = (Map<String, Object>) blockVariant.get("BlockCombination");
        blockFields.put("field_bool", null);
        expectEncodeFailure(nullBool, "null bool field", "Invalid field value for bool", "null is not allowed");

        HashMap<String, Object> nullI64 = (HashMap<String, Object>) deepCopy(valid);
        blockVariant = (Map<String, Object>) ((List<Object>) nullI64.get("blocks")).get(0);
        blockFields = (Map<String, Object>) blockVariant.get("BlockCombination");
        blockFields.put("field_i64", null);
        expectEncodeFailure(nullI64, "null i64 field", "Invalid field value for i64", "null is not allowed");

        HashMap<String, Object> nullU128 = (HashMap<String, Object>) deepCopy(valid);
        blockVariant = (Map<String, Object>) ((List<Object>) nullU128.get("blocks")).get(0);
        blockFields = (Map<String, Object>) blockVariant.get("BlockCombination");
        blockFields.put("field_u128", null);
        expectEncodeFailure(nullU128, "null u128 field", "Invalid field value for u128", "null is not allowed");

        HashMap<String, Object> f64NotBits = (HashMap<String, Object>) deepCopy(valid);
        blockVariant = (Map<String, Object>) ((List<Object>) f64NotBits.get("blocks")).get(0);
        blockFields = (Map<String, Object>) blockVariant.get("BlockCombination");
        blockFields.put("field_f64", Boolean.TRUE);
        expectEncodeFailure(
            f64NotBits,
            "f64 not represented as BigInteger bits",
            "Invalid field value for f64",
            "expected BigInteger bits");

        HashMap<String, Object> nullString = (HashMap<String, Object>) deepCopy(valid);
        Map<String, Object> payloadVariant = (Map<String, Object>) nullString.get("payload");
        Map<String, Object> payloadFields = (Map<String, Object>) payloadVariant.get("PayloadA");
        payloadFields.put("field_str", null);
        expectEncodeFailure(
            nullString,
            "null string field",
            "Invalid field value for string",
            "null is not allowed");

        HashMap<String, Object> nullVec = (HashMap<String, Object>) deepCopy(valid);
        payloadVariant = (Map<String, Object>) nullVec.get("payload");
        payloadFields = (Map<String, Object>) payloadVariant.get("PayloadA");
        payloadFields.put("vec_u8", null);
        expectEncodeFailure(
            nullVec,
            "null vec field",
            "Invalid field value for vec",
            "null is not allowed");
    }

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

        runCoverageProbes();

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
