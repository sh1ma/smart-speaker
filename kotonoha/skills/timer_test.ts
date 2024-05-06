import { Timer } from "./timer.ts"
import { assert } from "https://deno.land/std/assert/assert.ts"

Deno.test("タイマーがセットできるか", async () => {
  const timer = new Timer()
  const result = await timer._call(1000)
  assert(result === "1秒のタイマーをセットしました！")
})
