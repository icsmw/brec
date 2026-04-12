import { test, expect } from '@playwright/test';

test('browser client echoes binary packets from rust server', async ({ page }) => {
  let clientSummaryLine = null;
  page.on('console', (msg) => {
    const text = msg.text();
    console.log(`[browser:${msg.type()}] ${text}`);
    if (text.startsWith('CLIENT_SUMMARY ')) {
      clientSummaryLine = text;
    }
  });
  page.on('pageerror', (err) => {
    console.error('[browser:pageerror]', err);
  });
  page.on('requestfailed', (req) => {
    const failure = req.failure();
    console.error(
      `[browser:requestfailed] ${req.method()} ${req.url()} :: ${failure?.errorText ?? 'unknown'}`,
    );
  });

  const wsAddr = process.env.TEST_WS_ADDR || 'host.docker.internal:19001';
  const packageCount = Number(process.env.TEST_PACKAGE_COUNT || '100');
  const wsUrl = `ws://${wsAddr}`;

  console.log(`[e2e] goto client with ws=${wsUrl}, expect packets=${packageCount}`);
  await page.goto(`/?ws=${encodeURIComponent(wsUrl)}`);
  await expect(page.locator('#status')).toHaveText('done', { timeout: 120000 });

  const rxText = await page.locator('#rx').textContent();
  const txText = await page.locator('#tx').textContent();
  const rxBytesText = await page.locator('#rx-bytes').textContent();
  const txBytesText = await page.locator('#tx-bytes').textContent();
  const rx = Number((rxText ?? '0').replace(/[^0-9]/g, ''));
  const tx = Number((txText ?? '0').replace(/[^0-9]/g, ''));
  const rxBytes = Number((rxBytesText ?? '0').replace(/[^0-9]/g, ''));
  const txBytes = Number((txBytesText ?? '0').replace(/[^0-9]/g, ''));

  if (rx !== packageCount || tx !== packageCount) {
    const errText = await page.locator('#error').textContent();
    console.error(`[e2e] client error text: ${errText ?? ''}`);
  }

  expect(rx).toBe(packageCount);
  expect(tx).toBe(packageCount);
  expect(rxBytes).toBeGreaterThan(0);
  expect(txBytes).toBeGreaterThan(0);

  console.log(
    clientSummaryLine ??
      `CLIENT_SUMMARY packets_rx=${rx} packets_tx=${tx} bytes_rx=${rxBytes} bytes_tx=${txBytes}`,
  );
});
