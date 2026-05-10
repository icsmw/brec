FROM mcr.microsoft.com/playwright:v1.58.2-jammy

WORKDIR /workspace

ENV PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1

COPY generated/browser/npm ./generated/browser/npm
COPY clients/browser/package*.json ./clients/browser/
WORKDIR /workspace/clients/browser

RUN npm install

COPY clients/browser ./

CMD ["npm", "run", "test:e2e"]
