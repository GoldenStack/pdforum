#import "yagenda.typ": *
#show: agenda.with(
  name: "Pumpkins and peanuts committee",
  date: "03/01/2000", 
  time: "2 pm",
  location: "baseball field", 
  invited: [Sally, Shroeder, Pig-pen, Marcie]
)

// load external yaml
// #let topics = yaml("agenda.yaml")

// alternative: embed yaml in-place

#let tmp = `
"admin":
  Topic: Misc. admin
  Time: 5 mins
  Lead: baptiste 
  Purpose: Inform, Admin
  Preparation: |
    - Anything to change on the agenda? 
    - Read minutes from last meeting
    - Review action points
  Process: |
    - Check if agenda needs to be adjusted
    - Check if everyone's happy with the minutes
    - List top priorities of meeting and items needing decisions
    
"snoopy update":
  Topic: Snoopy's latest 
  Time: 10 mins
  Lead: Charlie Brown
  Purpose: Share, Discuss
  Preparation: |
    - Bring your favorite comic strip
    - Reflect on Snoopy's recent escapades
  Process: |
    - Discuss Snoopy's ongoing "battle" with the Red Baron
    - Share anecdotes of Snoopy's fiercest war moves
    - Plan a group outing to visit Snoopy's doghouse

"woodstock sighting":
  Topic: Woodstock's whereabouts
  Time: 5 mins
  Lead: Linus
  Purpose: Locate, Update
  Preparation: |
    - Bring binoculars for bird-watching
    - Brush up on bird calls
  Process: |
    - Share any recent sightings or chirpings from Woodstock
    - Discuss strategies for bringing Woodstock back to the tunes
    - Plan an upside-down birdhouse-building workshop 

"great pumpkin plans":
  Topic: Prep for Great Pumpkin
  Time: 15 mins
  Lead: Linus
  Purpose: Plan, Excite
  Preparation: |
    - Bring your pumpkin-carving tools
    - Practice your most sincere pumpkin patch speech
  Process: |
    - Discuss tactics for maximizing sincerity in the pumpkin patch
    - Brainstorm new ways to attract the Great Pumpkin's attention
    - Assign roles for the annual pumpkin carving contest

"philosophical discussion":
  Topic: Philosophical Musings
  Time: 20 mins
  Lead: Lucy and Charles
  Purpose: Ponder, Reflect
  Preparation: |
    - Bring your favorite existentialist quotes
    - Contemplate the meaning of life #v(100%)
  Process: |
    - Engage in deep philosophical discussions under the night sky
    - Debate the nature of happiness, existence, and 5c psychiatry
    - Seek wisdom from Linus's trusty security blanket

"beethoven appreciation":
  Topic: Beethoven's Legacy
  Time: 10 mins
  Lead: Schroeder
  Purpose: Appreciate, Discuss
  Preparation: |
    - Bring your favorite Beethoven compositions
    - Practice your air piano skills
  Process: |
    - Listen to Schroeder perform select Beethoven pieces
    - Discuss the timeless appeal of Beethoven's music
    - Plan a Beethoven-themed recital for Christmas

"baseball game":
  Topic: Baseball Game
  Time: 30 mins
  Lead: Charlie Brown
  Purpose: Play, Bond
  Preparation: |
    - Bring your baseball glove and bat
    - Review the rules of baseball
  Process: |
    - Divide into teams for an exciting game of baseball
    - Cheer on Charlie Brown as he attempts to finally kick that football
    - Enjoy snacks and camaraderie under the shade of the old oak tree
    
"summer camp":
  Topic: Summer Camp Adventures
  Time: 15 mins
  Lead: Peppermint Patty
  Purpose: Plan, Excite
  Preparation: |
    - Bring ideas for summer camp activities!
    - Check availability of camping gear!
  Process: |
    - Brainstorm camping trip destinations and outdoor activities
    - Discuss potential guest speakers or counselors
    - Organize a talent show and marshmallow roasting competition

"marcie school update":
  Topic: "{{ PDForum Template }}"
  Time: 10 mins
  Lead: Peppermint Patty
  Purpose: Discuss, Support
  Preparation: |
    - Bring Marcie's recent report card
    - Reflect on Marcie's study habits and strengths
  Process: |
    - Share updates on Marcie's academic achievements and challenges
    - Discuss strategies to help Marcie excel in school
    - Offer encouragement and support to Marcie in her studies

"linus blanket workshop":
  Topic: "{{ PDForum Template }}"
  Time: 20 mins
  Lead: Lucy
  Purpose: Understand, *Support*
  Preparation: |
    - Bring your own security blanket (optional)
    - Reflect on the significance of Linus's blanket
  Process: |
    - Discuss the history and symbolism of Linus's security blanket
    - Explore ways to help both Linus and Snoopy 
    - Release a blanket statement

"snoopy plane fights":
  Topic: "{{ PDForum Template }}"
  Time: 10 mins
  Lead: Woodstock
  Purpose: Support
  Preparation: |
    - Bring your favorite Snoopy flying ace memory
    - Reflect on Snoopy's aerial prowess
  Process: |
    - Share anecdotes from Snoopy's battles
    - Discuss the psychological implications of Snoopy's dogging of bullets
    `.text

#let topics = yaml.decode(tmp)

#agenda-table(topics)

#set page(flipped: false)

== Appendix

#lorem(120)
