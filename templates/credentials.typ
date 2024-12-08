#import "common.typ": *
#import "keyboard.typ": *

#let info = yaml("info.yml")

#show: common.with(info)

#let data = read("data.txt")

#{
    let username = info.field == "username";

    let title = if username {
      "ALIAS"
    } else {
      "PASSCODE"
    }

    let data = data.clusters().map(c => if username { c } else { $dot$ }).join(sym.zws);

    input_and_keyboard(title, data, info.url + info.path)
}
