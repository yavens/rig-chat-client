/**
 * @type {import("htmx.org").default}
 */
const HTMX = window.htmx;

// =================================
// Prompting
// =================================
/**
 * Reset the prompt box and remove the `No Chat History` placeholder
 * @param {Event} evt
 **/
function resetPrompt() {
  const promptInput = document.querySelector("#prompt");
  promptInput.value = "";

  document.querySelector("#placeholder")?.remove();
}

function scrollToResponse() {
  const chat = document.querySelector("#chat-history");
  chat.lastElementChild.scrollIntoView({ behavior: "smooth" });
}

// =================================
// Audio Recording
// =================================

/**
 * @type {MediaRecorder|null}
 **/
let recorder = null;
let interval = null;
let audioDataSegments = [];
let audioData = null;

/**
 * Initialize the recoder if necessary
 * and start the recording process
 */
async function startRecording() {
  if (recorder == null) {
    const mediaStream = await navigator.mediaDevices.getUserMedia({
      audio: true,
    });

    recorder = new MediaRecorder(mediaStream);

    // Collect chunks
    recorder.ondataavailable = (ev) => {
      audioDataSegments.push(ev.data);
    };
  }

  // Get data in 1 second chunks
  interval = setInterval(() => recorder.requestData(), 1000);

  audioDataSegments = [];
  recorder.start();
}

/**
 * Stop the recorder and perform the
 * required AJAX requests to the server
 */
async function stopRecording() {
  // Stop requesting data
  clearInterval(interval);

  // Wait for recorder to stop
  await new Promise((resolve, _reject) => {
    recorder.onstop = resolve;
    recorder?.stop();
  });

  // Create a blob from collected segments
  let blob = new Blob(audioDataSegments, { type: "audio/mp3" });

  // Convert the audio data to stringified json
  audioData = JSON.stringify([...new Uint8Array(await blob.arrayBuffer())]);

  // Make a request to the server to get the new prompt state
  HTMX.ajax("POST", "/api/recording", {
    target: "#recording-container",
    swap: "outerHTML",
    values: {
      data: audioData,
    },
  });

  // Once the new prompt state is received, send the preloaded prompt
  HTMX.on(
    "htmx:after-settle",
    function () {
      HTMX.trigger("#prompt-form", "submit");
    },
    { once: true }
  );
}

function getData() {
  return audioData;
}

// ============================
// Audio Playback
// ============================

/**
 * @typedef {Object} PlayAudioData
 * @property {string} data_uri - The URI of the audio data.
 * @property {number} generation_time - The time the audio was generated.
 */

/**
 * @type {[key: number]: PlayAudioData}
 */
const timeStampToAudio = {};

/**
 * @type {number[]}
 */
const audioQueue = []

let isAudioPlaying = false;

function playNextAudio() {
  if (isAudioPlaying) {
    return;
  }

  const timestamp = audioQueue.shift();

  if (timestamp === undefined) {
    return;
  }

  const audioData = timeStampToAudio[timestamp]

  delete timeStampToAudio[timestamp]

  const audio = new Audio(audioData.data_uri);
  audio.onended = () => {
    isAudioPlaying = false;
    playNextAudio();
  };

  isAudioPlaying = true;

  console.log("Playing:", timestamp)
  audio.play();
}

function playAudio(data) {
  /**
   * @type {PlayAudioData}
   */
  const audioData = JSON.parse(data);

  timeStampToAudio[audioData.generation_time] = audioData
  console.log("Received:", audioData.generation_time)

  if (audioData.generation_time == audioQueue[0]) {
    playNextAudio()
  }
}

function queueAudio(data) {
  const timestamp = Number.parseInt(data)
  console.log(timestamp)

  audioQueue.push(timestamp)

  audioQueue.sort((a,b) => a - b)
}

/**
 * Play the audio from the sse event
 * @param {CustomEvent} ev
 */
function sseBeforeMessage(ev) {
  /**
   * @type {MessageEvent}
   */
  const message = ev.detail;

  if (message.type == "play_audio") playAudio(message.data);
  else if (message.type == "queue_audio") queueAudio(message.data)

}

HTMX.on("htmx:sseBeforeMessage", sseBeforeMessage);
