FROM mcr.microsoft.com/playwright:v1.58.2-jammy

WORKDIR /workspace

ENV PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1

COPY clients/browser/package*.json ./clients/browser/
COPY binding/pkg ./binding/pkg
WORKDIR /workspace/clients/browser

RUN npm ci

COPY clients/browser ./

CMD ["npm", "run", "test:e2e"]
