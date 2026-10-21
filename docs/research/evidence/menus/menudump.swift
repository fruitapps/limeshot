import ApplicationServices
import AppKit
// usage: menudump <pid>  — dumps the native menu bar tree as indented text
let pid = pid_t(CommandLine.arguments[1])!
let app = AXUIElementCreateApplication(pid)
func attr(_ e: AXUIElement, _ n: String) -> AnyObject? { var v: AnyObject?; AXUIElementCopyAttributeValue(e, n as CFString, &v); return v }
func kids(_ e: AXUIElement) -> [AXUIElement] { (attr(e, kAXChildrenAttribute) as? [AXUIElement]) ?? [] }
func mods(_ m: Int) -> String { var s = ""; if m & 4 != 0 { s += "Ctrl+" }; if m & 2 != 0 { s += "Opt+" }; if m & 1 != 0 { s += "Shift+" }; if m & 8 == 0 { s += "Cmd+" }; return s }
func dump(_ e: AXUIElement, _ depth: Int) {
  for k in kids(e) {
    let role = attr(k, kAXRoleAttribute) as? String ?? ""
    if role == "AXMenu" { dump(k, depth); continue }
    let t = attr(k, kAXTitleAttribute) as? String ?? ""
    var line = String(repeating: "  ", count: depth) + (t.isEmpty ? "----" : t)
    if let c = attr(k, "AXMenuItemCmdChar") as? String, !c.isEmpty { line += "\t[" + mods((attr(k, "AXMenuItemCmdModifiers") as? Int) ?? 0) + c + "]" }
    let ch = attr(k, "AXMenuItemCmdChar") as? String ?? ""
    let vk = attr(k, "AXMenuItemCmdVirtualKey") as? Int ?? -1
    let gl = attr(k, "AXMenuItemCmdGlyph") as? Int ?? -1
    if !ch.isEmpty && ch.unicodeScalars.first!.value > 0x7f || vk > 0 || gl > 0 { line += "\t<chars=" + ch.unicodeScalars.map { String(format: "%X", $0.value) }.joined(separator: ",") + " vk=\(vk) glyph=\(gl) mods=\((attr(k, "AXMenuItemCmdModifiers") as? Int) ?? -1)>" }
    if let mk = attr(k, "AXMenuItemMarkChar") as? String, !mk.isEmpty { line += "\t{" + mk + "}" }
    if let en = attr(k, kAXEnabledAttribute) as? Bool, !en { line += "\t(disabled)" }
    print(line)
    dump(k, depth + 1)
  }
}
if let bar = attr(app, kAXMenuBarAttribute) { dump(bar as! AXUIElement, 0) } else { print("no menubar") }
