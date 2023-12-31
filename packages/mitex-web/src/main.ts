import './style.css'
import van from "vanjs-core"
import { convert_math } from "mitex-web-wasm"
const { div, textarea } = van.tags

const App = () => {
  const input = textarea({ placeholder: "Type LaTeX math equations here", autofocus: true, rows: 10 })
  const output = textarea({ readOnly: true, placeholder: "Output", rows: 10 })
  const error = div({ class: "error" })
  input.oninput = () => {
    try {
      output.value = convert_math(input.value, new Uint8Array)
      error.textContent = ""
    } catch (e) {
      output.value = ""
      error.textContent = e as string
    }
  }
  output.onfocus = () => output.select()
  return div(
    input,
    output,
    error
  )
}

van.add(document.querySelector("#app")!, App())
