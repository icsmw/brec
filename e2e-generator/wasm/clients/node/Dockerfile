FROM node:22-bookworm-slim

WORKDIR /workspace/clients/node

COPY generated/node/npm ../../generated/node/npm
COPY clients/node/package*.json ./
RUN npm install

COPY clients/node/dist ./dist

CMD ["npm", "run", "test:e2e"]
