# ─── Build stage: compile the Angular app ───
FROM node:20-alpine AS build

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci

COPY . .
RUN npm run build

# ─── Runtime stage: nginx serves the SPA + proxies the backends ───
FROM nginx:1.27-alpine

RUN rm /etc/nginx/conf.d/default.conf
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/dist/wedding-quest/browser /usr/share/nginx/html

EXPOSE 80
