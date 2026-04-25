using System.Net.WebSockets;
using System.Text;

namespace BrecE2e;

internal static class Program
{
    private static async Task<ArraySegment<byte>?> ReceiveBinaryAsync(ClientWebSocket ws, MemoryStream frameBuffer, CancellationToken ct)
    {
        frameBuffer.SetLength(0);
        var chunk = new byte[64 * 1024];

        while (true)
        {
            var result = await ws.ReceiveAsync(chunk, ct);
            if (result.MessageType == WebSocketMessageType.Close)
            {
                return null;
            }
            if (result.MessageType != WebSocketMessageType.Binary)
            {
                if (result.EndOfMessage)
                {
                    return ArraySegment<byte>.Empty;
                }
                continue;
            }

            if (result.Count > 0)
            {
                frameBuffer.Write(chunk, 0, result.Count);
            }

            if (result.EndOfMessage)
            {
                return new ArraySegment<byte>(frameBuffer.GetBuffer(), 0, (int)frameBuffer.Length);
            }
        }
    }

    public static async Task<int> Main(string[] args)
    {
        var wsAddr = Environment.GetEnvironmentVariable("TEST_WS_ADDR") ?? "host.docker.internal:19001";
        var expectedCountRaw = Environment.GetEnvironmentVariable("TEST_PACKAGE_COUNT") ?? "1000";
        if (!int.TryParse(expectedCountRaw, out var expectedCount))
        {
            Console.Error.WriteLine($"[client] invalid TEST_PACKAGE_COUNT: {expectedCountRaw}");
            return 2;
        }

        var uri = new Uri($"ws://{wsAddr}");
        using var ws = new ClientWebSocket();
        using var cts = new CancellationTokenSource(TimeSpan.FromMinutes(10));
        using var frameBuffer = new MemoryStream(128 * 1024);

        try
        {
            await ws.ConnectAsync(uri, cts.Token);
            Console.WriteLine("[client] connected");

            var packetCount = 0;
            long recvBytes = 0;
            long sentBytes = 0;

            while (true)
            {
                var frame = await ReceiveBinaryAsync(ws, frameBuffer, cts.Token);
                if (frame == null)
                {
                    break;
                }
                if (frame.Value.Count == 0)
                {
                    continue;
                }

                var input = new byte[frame.Value.Count];
                Buffer.BlockCopy(frame.Value.Array!, frame.Value.Offset, input, 0, input.Length);
                recvBytes += input.Length;

                using var packet = ClientBindings.DecodePacket(input);
                var output = ClientBindings.EncodePacket(packet);

                using var packetCheck = ClientBindings.DecodePacket(output);
                var outputCheck = ClientBindings.EncodePacket(packetCheck);
                if (!output.AsSpan().SequenceEqual(outputCheck))
                {
                    throw new InvalidOperationException("client reencode mismatch");
                }

                await ws.SendAsync(output, WebSocketMessageType.Binary, true, cts.Token);
                sentBytes += output.Length;
                packetCount += 1;

                if (packetCount % 10 == 0 || packetCount == expectedCount)
                {
                    Console.WriteLine($"[client] progress: {packetCount}/{expectedCount}");
                }
            }

            if (packetCount != expectedCount)
            {
                Console.WriteLine(
                    $"CLIENT_SUMMARY packets={packetCount} sent_bytes={sentBytes} recv_bytes={recvBytes} verified=false expected={expectedCount}");
                return 1;
            }

            Console.WriteLine(
                $"CLIENT_SUMMARY packets={packetCount} sent_bytes={sentBytes} recv_bytes={recvBytes} verified=true");
            return 0;
        }
        catch (Exception err)
        {
            Console.Error.WriteLine($"[client] error: {err.Message}");
            return 1;
        }
    }
}
