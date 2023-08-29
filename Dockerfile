FROM rust:1.68

WORKDIR /usr/src/myapp
COPY . .

RUN cargo install --path .

EXPOSE 8082/tcp

CMD ["git-pages"]
