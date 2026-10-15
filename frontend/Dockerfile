# The SvelteKit frontend (adapter-node).
FROM node:22-alpine AS build
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
RUN npm run build && npm prune --omit=dev

FROM node:22-alpine
WORKDIR /app
COPY --from=build /app/build ./build
COPY --from=build /app/node_modules ./node_modules
COPY --from=build /app/package.json ./
ENV PORT=3001
# The commit, for browser error reports to name the build. CI passes it; unset without.
ARG GIT_SHA=
ENV PUBLIC_SENTRY_RELEASE=${GIT_SHA:+wormholesystems-web@$GIT_SHA}
EXPOSE 3001
CMD ["node", "build"]
