import OpenAI from "npm:openai"
import { toFile } from "npm:openai"
import { Hono } from "https://deno.land/x/hono@v4.2.4/mod.ts"
import { AgentExecutor, createOpenAIFunctionsAgent } from "npm:langchain/agents"
import type { ChatPromptTemplate } from "npm:@langchain/core/prompts"
import { pull } from "npm:langchain/hub"

import { ChatOpenAI } from "npm:@langchain/openai"

import { Tool } from "npm:@langchain/core/tools"
import { Timer } from "./skills/timer.ts"

/**
 * The Calculator class is a tool used to evaluate mathematical
 * expressions. It extends the base Tool class.
 * @example
 * ```typescript
 * const calculator = new Calculator();
 * const sum = calculator.add(99, 99);
 * console.log("The sum of 99 and 99 is:", sum);
 * ```
 */
export class TextCounter extends Tool {
  static lc_name() {
    return "TextCounter"
  }

  get lc_namespace() {
    return [...super.lc_namespace, "text_counter"]
  }

  name = "text_counter"

  // deno-lint-ignore require-await
  async _call(input: string) {
    return input.length.toString()
  }

  description = `文字数のカウントを行います。回答は日本語で行います`
}

const apiKey = Deno.env.get("OPENAI_API_KEY")

if (!apiKey) {
  throw new Error("OPENAI_API_KEY is not set")
}

const llm = new ChatOpenAI({
  apiKey: apiKey,
})

const prompt = await pull<ChatPromptTemplate>(
  "hwchase17/openai-functions-agent"
)

const tools = [new TextCounter(), new Timer()]

export const agent = await createOpenAIFunctionsAgent({
  llm,
  tools,
  prompt,
})

export const agentExecutor = new AgentExecutor({
  agent,
  tools,
})

const openai = new OpenAI({
  apiKey,
})

const app = new Hono()

app.get("/ping", (c) => c.json("pong"))
app.post("/transcribe", async (c) => {
  const file = await toFile(c.req.blob(), "audio.wav")
  const resp = await openai.audio.transcriptions.create({
    model: "whisper-1",
    file: file,
    language: "ja",
  })
  return c.json(resp)
})

app.post("/talk", async (c) => {
  console.log(await c.req.text())
  const { text } = await c.req.json()
  const chatResp = await agentExecutor.invoke({ input: text })
  // console.log(chatResp)
  return c.json(chatResp)
})

app.post("/ask", async (c) => {
  const wav = await c.req.blob()
  const audio = await handleAsk(wav)
  return c.json(audio)
})

// Learn more at https://deno.land/manual/examples/module_metadata#concepts
if (import.meta.main) {
  Deno.serve({ port: 8081 }, app.fetch)
}

export const handleAsk = async (wav: Blob) => {
  const transcription = await transcribeFromWav(wav)
  const text = transcription.text
  const talkResult = await talkToAgent(text)

  try {
    console.log(`音声を生成: ${talkResult.output}`)
    const audio = await synthesis(talkResult.output, "3", { speedScale: "1.3" })
    return audio
  } catch (e) {
    console.error(e)
    throw new Error("Failed to synthesize audio")
  }
}

const transcribeFromWav = async (wav: Blob) => {
  const file = await toFile(wav, "audio.wav")
  return await openai.audio.transcriptions.create({
    model: "whisper-1",
    file: file,
    language: "ja",
  })
}

const talkToAgent = async (prompt: string) => {
  return await agentExecutor.invoke({ input: prompt })
}

// Voicevox

const voicevoxURL = "http://localhost:50021"

type GetVoicevoxAudioQueryParams = {
  speaker: string
  text: string
}

const getVoicevoxAudioQuery = async ({
  speaker,
  text,
}: GetVoicevoxAudioQueryParams) => {
  const query = new URLSearchParams({ speaker, text })
  const endpointWithQuery = new URL(`${voicevoxURL}/audio_query?${query}`)

  const resp = await fetch(endpointWithQuery, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
  })
  return await resp.json()
}

type VoicevoxAudioQuery = {
  speedScale: string
}

export const synthesis = async (
  text: string,
  speakerId: string,
  customAudioQuery: VoicevoxAudioQuery
) => {
  const audioQuery = await getVoicevoxAudioQuery({ speaker: "3", text })
  const customizedAudioquery = { ...audioQuery, ...customAudioQuery }

  const synthesisRequestQuery = new URLSearchParams({ speaker: speakerId })
  const endpointWithQuery = new URL(
    `${voicevoxURL}/synthesis?${synthesisRequestQuery}`
  )

  const resp = await fetch(endpointWithQuery, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(customizedAudioquery),
  })

  if (!resp.ok) {
    throw new Error("Failed to synthesis audio")
  }

  return await resp.blob()
}
