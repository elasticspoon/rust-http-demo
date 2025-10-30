FROM nginx:trixie-perl

COPY nginx-static /usr/share/nginx/html

CMD ["nginx-debug", "-g", "daemon off;"]
