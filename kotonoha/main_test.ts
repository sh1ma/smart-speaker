import { assertEquals } from "https://deno.land/std@0.224.0/assert/mod.ts"
import { agentExecutor, handleAsk, synthesis } from "./main.ts"
import { join } from "https://deno.land/std@0.224.0/path/mod.ts"

Deno.test("タイマーセットができる", async () => {
  const input = "5秒のタイマーをセットして"
  await agentExecutor.invoke({ input })
})

Deno.test("音声が合成できる", async () => {
  type TestCase = {
    input: string
  }

  const testCases: TestCase[] = [
    { input: "5秒のタイマーをセットして" },
    { input: "10秒のタイマーをセットして" },
    { input: "15秒のタイマーをセットして" },
  ]

  for (const { input } of testCases) {
    const result = await synthesis(input, "3", { speedScale: "1.3" })
    assertEquals(result.type, "audio/wav")
  }
})

Deno.test(
  "音声データを受け取って回答を生成し、回答の音声ファイルを返す",
  async () => {
    const inputWav = await Deno.readFile(
      join(Deno.cwd(), "testdata", "test.wav")
    )

    const result = await handleAsk(
      new Blob([inputWav], {
        type: "audio/wav",
      })
    )
    assertEquals(result.type, "audio/wav")
  }
)
