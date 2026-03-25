import { createApp } from "vue";
import App from "./App.vue";
import { createLogger } from "./composables/useLogger";

const log = createLogger("app");

const app = createApp(App);

app.config.errorHandler = (err, _instance, info) => {
  log.error(`Vue error [${info}]: ${err}`);
};

window.addEventListener("unhandledrejection", (event) => {
  log.error(`Unhandled rejection: ${event.reason}`);
});

app.mount("#app");
