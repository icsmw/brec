FROM node:22-bookworm-slim

WORKDIR /workspace/client

COPY generated/npm ../generated/npm
COPY client/package*.json ./
RUN npm install

COPY client/dist ./dist

CMD ["npm", "run", "test:e2e"]
