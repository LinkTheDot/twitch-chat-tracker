# Stage 1: Build the React application
FROM node:20-alpine AS build

WORKDIR /app

COPY package.json yarn.lock* ./
RUN npm install --frozen-lockfile

COPY . .

RUN npm run build

# Stage 2: Serve the built application with Nginx
FROM nginx:alpine AS production

# Copy the build output from the previous stage
COPY --from=build /app/dist /usr/share/nginx/html

# Expose port 80 for the web server
EXPOSE 80

# Start Nginx
CMD ["nginx", "-g", "daemon off;"]
