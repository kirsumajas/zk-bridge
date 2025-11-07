FROM node:18-alpine

WORKDIR /app

# Copy package files
COPY package*.json ./
COPY packages/relayer/package.json ./packages/relayer/
COPY packages/submission-manager/package.json ./packages/submission-manager/
COPY packages/sdk/package.json ./packages/sdk/

# Install dependencies
RUN npm install

# Copy source code
COPY . .

# Build the application
RUN npm run build

EXPOSE 3000 3001

CMD ["npm", "start"]