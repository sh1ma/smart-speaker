import { Tool } from "npm:@langchain/core/tools"
import { difference } from "https://deno.land/std@0.224.0/datetime/difference.ts"

export class Timer extends Tool {
  static lc_name() {
    return "Timer"
  }

  get lc_namespace() {
    return [...super.lc_namespace, "timer"]
  }

  name = "timer"

  // deno-lint-ignore require-await
  async _call(ms: number): Promise<string> {
    const now = new Date()
    const after = new Date(now.getTime() + ms)
    const diff = difference(after, now)
    const hourStr = diff.hours === 0 ? "" : `${diff.hours}時間`
    const minuteStr = diff.minutes === 0 ? "" : `${diff.minutes}分`
    const secondStr = diff.seconds === 0 ? "" : `${diff.seconds}秒`
    const timeStr = `${hourStr}${minuteStr}${secondStr}`

    return `${timeStr}のタイマーをセットしました！`
  }

  description = `指定された時間のタイマーをセットします。`
}
