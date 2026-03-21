// Ring-rs Demo Stub — 点击挑战小游戏

const GAME_DURATION = 10; // 秒
const TARGET_SCORE = 10;

let score = 0;
let timeLeft = GAME_DURATION;
let gameOver = false;
let timerInterval = null;

const scoreEl = document.getElementById("score");
const timerFill = document.getElementById("timer-fill");
const targetEl = document.getElementById("target-score");
const clickBtn = document.getElementById("click-btn");
const gameArea = document.getElementById("game-area");
const resultArea = document.getElementById("result-area");
const finalScoreEl = document.getElementById("final-score");
const resultMsgEl = document.getElementById("result-msg");
const returnBtn = document.getElementById("return-btn");

targetEl.textContent = TARGET_SCORE;

function postToEngine(message) {
  const payload = JSON.stringify(message);
  if (window.ipc && typeof window.ipc.postMessage === "function") {
    window.ipc.postMessage(payload);
    return true;
  }
  if (window.engine && typeof window.engine.onComplete === "function" && message.type === "onComplete") {
    window.engine.onComplete(message.result);
    return true;
  }
  console.log("IPC unavailable, message:", payload);
  return false;
}

function updateTimer() {
  timeLeft -= 0.1;
  if (timeLeft <= 0) {
    timeLeft = 0;
    endGame();
  }
  timerFill.style.width = ((timeLeft / GAME_DURATION) * 100) + "%";
}

function endGame() {
  if (gameOver) return;
  gameOver = true;
  clearInterval(timerInterval);

  gameArea.style.display = "none";
  resultArea.classList.add("show");
  finalScoreEl.textContent = score;

  if (score >= TARGET_SCORE) {
    resultMsgEl.textContent = "挑战成功！";
    resultMsgEl.style.color = "#44ff88";
  } else {
    resultMsgEl.textContent = "再接再厉！";
    resultMsgEl.style.color = "#ff8844";
  }
}

clickBtn.addEventListener("click", () => {
  if (gameOver) return;
  score++;
  scoreEl.textContent = score;
  scoreEl.classList.add("bump");
  setTimeout(() => scoreEl.classList.remove("bump"), 100);

  if (score >= TARGET_SCORE) {
    endGame();
  }
});

returnBtn.addEventListener("click", () => {
  const result = score >= TARGET_SCORE ? "win" : "lose";
  postToEngine({ type: "onComplete", result: result });
});

timerInterval = setInterval(updateTimer, 100);
