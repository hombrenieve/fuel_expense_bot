FROM node:18-alpine

WORKDIR /usr/src/app
COPY src/* package.json ./
RUN npm install

CMD ["bot.js"]