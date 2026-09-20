# Builds the React front end and serves it with nginx, which also fronts the
# API so the browser only ever sees one origin.
#
#   docker build -t water-sort-frontend frontend
#   docker run --rm -p 5173:80 -e API_URL=http://api:8080 water-sort-frontend

FROM node:22-alpine AS builder

WORKDIR /build

# The lockfile alone first, so editing a component does not reinstall the world.
COPY package.json package-lock.json ./
RUN npm ci

COPY . .
RUN npm run build

FROM nginx:1.27-alpine AS runtime

# `API_URL` is substituted into the nginx config at start up, so the same image
# works next to a container named `api` or in front of an API somewhere else.
ENV API_URL=http://api:8080

COPY nginx.conf.template /etc/nginx/templates/default.conf.template
COPY --from=builder /build/dist /usr/share/nginx/html

EXPOSE 80
