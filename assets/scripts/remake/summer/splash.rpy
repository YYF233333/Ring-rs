image splash:
    "RING_splash.png"
    size (1920,1080)
    anchor (0.5,1.0)


label splashscreen:
    scene black
    with Pause(0.5)

    show splash with dissolve
    with Pause(2)

    scene black with dissolve
    with Pause(1.0)

    return

# label splashscreen:
#     scene black
#     with Pause(1)

#     show text "American Bishoujo Presents..." with dissolve
#     with Pause(2)

#     hide text with dissolve
#     with Pause(1)

#     return

# label splashscreen:

#     $ renpy.movie_cutscene('[Machikado Mazoku S2][01][BIG5][1080P] (video-converter.com).mpg')

#     return