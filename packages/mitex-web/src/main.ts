import './style.css'
import van from "vanjs-core"
import { convert_math } from "mitex-web-wasm"
const { div, textarea, input } = van.tags

const App = () => {
  const input_area = textarea({ placeholder: "Type LaTeX math equations here", autofocus: true, rows: 10 })
  const template_checkbox = input({ checked: true, type: "checkbox", name: "With typst templates" })
  const import_checkbox = input({ checked: false, type: "checkbox", name: "With mitex imports" })
  const checkbox_container = div(
    div(
      template_checkbox,
      "With typst templates"
    ),
    div(
      import_checkbox,
      "With mitex imports"
    )
  )
  const output = textarea({ readOnly: true, placeholder: "Output", rows: 10 })
  const error = div({ class: "error" })
  const update_output = () => {
    try {
      let convert_res = convert_math(input_area.value, new Uint8Array)
      if (template_checkbox.checked) {
        convert_res = `#math.equation(eval("$" + "${convert_res}" + "$", scope: mitex-scope))`
      }
      if (import_checkbox.checked) {
        convert_res = `#import "@preview/mitex:0.1.0": *` + "\n\n" + convert_res
      }
      output.value = convert_res
      error.textContent = ""
    } catch (e) {
      output.value = ""
      error.textContent = e as string
    }
  }
  output.onfocus = () => output.select()
  input_area.oninput = update_output
  template_checkbox.onchange = update_output
  import_checkbox.onchange = update_output
  return div(
    input_area,
    output,
    checkbox_container,
    error
  )
}

van.add(document.querySelector("#app")!, App())
