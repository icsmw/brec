FROM node:22-bookworm-slim

WORKDIR /workspace

COPY clients/node/package*.json ./clients/node/
COPY binding/pkg ./binding/pkg
WORKDIR /workspace/clients/node

RUN npm install

COPY clients/node/src ./src

CMD ["npm", "run", "test:e2e"]
