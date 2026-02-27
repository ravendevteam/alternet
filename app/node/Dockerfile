FROM debian:bookworm-slim
WORKDIR /app
COPY app /app/node
RUN chmod +x /app/node
ENTRYPOINT ["/app/node"]