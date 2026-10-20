import { mount } from "svelte";
import "../shared/tokens.css";
import "../shared/base.css";
import { backend } from "$backend";
import App from "./App.svelte";

const target = document.getElementById("app");
if (!target) throw new Error("the page has no #app element");
// Every screen reaches the backend with getContext("backend").
mount(App, { target, context: new Map([["backend", backend]]) });
