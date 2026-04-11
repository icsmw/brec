FROM mcr.microsoft.com/playwright:v1.58.2-jammy

WORKDIR /workspace

ENV PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1

COPY client/package*.json ./client/
COPY bindings/pkg ./bindings/pkg
WORKDIR /workspace/client

RUN npm ci

COPY client ./

CMD ["npm", "run", "test:e2e"]
