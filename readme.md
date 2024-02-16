# todo-actix ~ utoipa with utoipa-swagger-ui, utoipa-redoc and utoipa-rapidoc example

This is a demo `actix-web` application with in-memory storage to manage Todo items. The API
demonstrates `utoipa` with `utoipa-swagger-ui` functionalities.

For security restricted endpoints the super secret API key is: `utoipa-rocks`.

Just run command below to run the demo application and browse to `http://localhost:8080/swagger-ui/`.

If you prefer Redoc just head to `http://localhost:8080/redoc` and view the Open API.

RapiDoc can be found from `http://localhost:8080/rapidoc`.

```bash
cargo run
```

If you want to see some logging, you may prepend the command with `RUST_LOG=debug` as shown below.

```bash
RUST_LOG=debug cargo run
```


You can also use cargo watch to automatically reload the application whenever changes are made to the code. To do this, ensure you have cargo watch installed and run the following command:

```bash
cargo watch -x 'run'
```