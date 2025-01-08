#import "common.typ": *
#import "keyboard.typ": *

#let info = yaml("info.yml")

#show: common.with(info)

#let data = read("data.txt");

#let base-url = info.url + "/publish/"

#set document(title: "posting as " + info.username)

#align(center, {
  v(2.5%) + h(5.5pt)

  text(size: 32pt, fill: luma(80), smallcaps("posting as " + info.username))
  
})

#align(center, align(left, {
  v(3%)
  
  text(size: 18pt, fill: luma(80))[MESSAGE] + linebreak()
  
  text-box-next(data, base-url)

  v(3%)

  keyboard(base-url)
}))
