FROM mcr.microsoft.com/playwright:v1.55.0-jammy

WORKDIR /workspace

COPY . .

WORKDIR /workspace/client

RUN npm install
RUN npx playwright install chromium

CMD ["npm", "run", "test:e2e"]
