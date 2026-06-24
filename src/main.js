// Tauri Screen Veil 前端逻辑
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const statusDot = document.getElementById("status-dot");
const statusText = document.getElementById("status-text");
const toggleBtn = document.getElementById("toggle-btn");
const quitBtn = document.getElementById("quit-btn");
const textInput = document.getElementById("text-input");
const bgInput = document.getElementById("bg-input");
const fgInput = document.getElementById("fg-input");

// 同步状态显示
function setStatus(visible) {
  if (visible) {
    statusDot.classList.add("on");
    statusText.textContent = "显示中";
  } else {
    statusDot.classList.remove("on");
    statusText.textContent = "隐藏";
  }
}

// 监听后端状态变更
listen("veil-state-changed", (event) => {
  setStatus(event.payload.visible);
});

// 初始化: 查询当前状态
invoke("get_veil_state").then((state) => {
  setStatus(state.visible);
});

// 按钮: 手动切换
toggleBtn.addEventListener("click", async () => {
  await invoke("toggle_veil");
});

// 按钮: 退出
quitBtn.addEventListener("click", async () => {
  await invoke("quit_app");
});

// 文字修改
let textTimer = null;
textInput.addEventListener("input", () => {
  clearTimeout(textTimer);
  textTimer = setTimeout(async () => {
    await invoke("set_veil_text", { text: textInput.value });
  }, 300);
});

let bgTimer = null;
bgInput.addEventListener("input", () => {
  clearTimeout(bgTimer);
  bgTimer = setTimeout(async () => {
    await invoke("set_veil_bg", { color: bgInput.value });
  }, 300);
});

let fgTimer = null;
fgInput.addEventListener("input", () => {
  clearTimeout(fgTimer);
  fgTimer = setTimeout(async () => {
    await invoke("set_veil_fg", { color: fgInput.value });
  }, 300);
});
