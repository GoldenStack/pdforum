#let data = {
  let data = read("data.txt")
  let start = data.position("\n")

  (data.slice(0, start), data.slice(start))
}

#align(center + horizon)[
  #text(size: 10em)[
    #data.at(0)
  ]

  #v(-10em)
  
  #text(size: 4em)[
    #data.at(1)
  ]
]