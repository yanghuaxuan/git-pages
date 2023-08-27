FROM rust:1.67

WORKDIR /usr/src/myapp
COPY . .

RUN cargo install --path .

EXPOSE 8082/tcp

CMD ["git-pages"]
