import { mount } from "svelte";
import "../shared/tokens.css";
import "../shared/base.css";
import App from "./App.svelte";

const target = document.getElementById("app");
if (!target) throw new Error("the page has no #app element");
mount(App, { target });
