#import "common.typ": *
#import "keyboard.typ": *

#let info = yaml("info.yml")

#show: common.with(info)

#let data = read("data.txt")

#let username = "" + data.split("\u{0}").at(0);
#let password = "" + data.split("\u{0}").at(1).clusters().map(c => $dot$).join(sym.zws);

#let action = if info.type == "register" {
  "create an account"
} else {
  "sign in"
}

#let base-url = info.url + "/" + info.type + "/";

#set document(title: action)

#align(center, {
  v(2.5%) + h(5.5pt)

  text(size: 32pt, fill: luma(80), smallcaps(action))
  
  v(1%) + h(5.5pt)
  text(size: 18pt)[to continue to PDForum]
  
})

#align(center, align(left, {
  v(3%)
  
  text(size: 18pt, fill: luma(80))[ALIAS] + linebreak()
  
  text-box-next(username, base-url, selected: info.field == "username")

  v(3%)
  
  text(size: 18pt, fill: luma(80))[PASSCODE] + linebreak()
  
  text-box-next(password, base-url, selected: info.field == "password")

  v(3%)

  keyboard(base-url)
}))
