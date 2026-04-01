FROM node:22-bookworm-slim

WORKDIR /workspace/client

COPY client/package*.json ./
RUN npm install

COPY client/src ./src
COPY client/native ./native

CMD ["npm", "run", "test:e2e"]
