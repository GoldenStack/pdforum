#import "common.typ": *

#let info = yaml("info.yml")

#show: common.with(info)

#set text(14pt)

#{
  let data = read("data.txt").split("\u{0}")
  let (author, likes, comments, publish_time, content) = data

  let post = {
    v(1em)
    link(info.url + "/user/" + author, strong(author))
    h(0.5em)
    sym.dot
    h(0.5em)
    smallcaps(text(fill: luma(100), publish_time))
    
    v(0em)
    content

    align(horizon, grid(
      columns: (auto, 2em, auto, 5fr),
      rows: (20pt),
      column-gutter: 3pt,
      image("svg/heart.svg"),
      likes,
      image("svg/comment.svg"),
      comments,
    ))
  }
  
  align(center, box(width: 23em, align(left, post)))
}