label chapter20:
    #第三幕第七场
    scene bg yiyuan_bingfang_ with tran_anticlockwise()
    旁白 "3日后，普通病房"
    show hongye pajamas_normal1 at s11
    红叶 "似乎把你吓得不轻啊…"
    子文 "拜此所赐，我三天只睡了不到七个小时，但是我完成了这个。"
    旁白 "我苦笑着递出剧本稿纸。"
    show hongye pajamas_normal4 at nod
    红叶  "是这些吗？"
    子文 "其实，结局我还没写完。虽然我答应你要写幸福的GE，但是当我落笔的时候，我发现我找不到道路了。"
    红叶 pajamas_normal1 "是这样吗…"
    show hongye pajamas_normal1 at s11("close")
    旁白 "红叶突然抱住我。"
    play music "audio/BGM/15.夕陽～君に幸あれ～.mp3"
    子文 "这么突然…"
    红叶 pajamas_backhand5 "再多一下下就好…"
    子文 "没事的，我现在就在这里，你想多久都可以。"
    红叶 pajamas_backhand6 "我真的好害怕。好黑，好黑，好黑。感觉不到自己的身体，像是沉入了幽暗的海底，并且一直一直地下沉着。"
    子文 "…"
    旁白 "我们感受着彼此的体温。在无边的黑暗之中，我们彼此温暖着。即便看不见前方，即便心绪难以传达给所有人，但是现在，我们只拥有彼此了。"
    红叶 pajamas_backhand1 "我现在把第二个愿望告诉你…"
    旁白 "红叶松开手，看着我的眼睛。"
    红叶 pajamas_normal2 "我要为你的剧本配音。当然，你自己来配男主。"
    子文 "诶？"
    红叶 pajamas_normal1 "所以…我要接受手术。但是在那之前，还剩下的三天时间，请让我再试一次。"
    红叶 "我不再是一个声优，我没有高超的技术，失去了坚实的体能，但是我无论如何，无论如何也想再试一次。"
    红叶 pajamas_normal2 "不只是因为我们说好的事情，更因为，这个故事，我也是创作者！"
    子文 "但是你的身体真的允许吗…我有点放心不下。"
    红叶 pajamas_backhand1 "即便只有几个小时，我还有一息尚存，我还能说话，我就要继续录音。这里不是录音室，我的状况也可能会很麻烦，"
    红叶 "但是子文，哪怕搀着我也好，请让我继续下去。我，我的一切，现在交给你了。"
    旁白 "..."
    hide hongye 
    scene black with dissolve
    

    ###以下为场景切换##
    scene bg yiyuan_bingfang_y with dissolve
    旁白 "…"
    红叶 pajamas_backhand6 "唔…"
    子文 "休息一会吧。这一段长句比较多…"
    红叶 pajamas_normal1 "没事的…"
    旁白 "…"
    红叶 pajamas_backhand3 "老问题了啊，你这人，怎么又写棒读句？"
    子文 "啊啊啊我错了我错了…"
    
    scene bg yiyuan_bingfang_ with dissolve
    旁白 "清晨"
    子文 "休息的如何？今天我也带了蜂蜜蛋糕。"
    红叶 pajamas_normal5 "嗯，我休息的很好。今天也要继续努力。"

    layeredimage temp:
        group bg:
            zoom 0.5
            attribute yiyuanlouding_n:
                "images/bg/医院/医院楼顶/bg053_sashool_n_19201440.webp"
            attribute yekong:
                "images/bg/天空与雪景/夜空.webp"
    
    scene temp yiyuanlouding_n:
        yalign 0.0
    with dissolve
    旁白 "…"
    旁白 "夜晚，医院楼顶"
    子文 "这么冷的天非要上来看星星…"
    红叶 pajamas_backhand5 "子文身上很暖和啊！"
    show bg yekong:
        yalign 1.0
        alpha 0.0
        parallel:
            linear 2.0 alpha 1.0
        parallel:
            easeout 0.5 yalign 0.95
            linear 3.5 yalign 0.05
            easein 0.5 yalign 0.0
    子文 "…你以前可从来不要我背着你的啊？喂，把衣服披好！」{w=3}{nw}" (what_suffix = "")

    scene black with Dissolve(1.0)
    旁白 "…"
    旁白 "……"
    旁白 "………"
    #TODO:音乐渐响（这里用啥音乐@wzy
    scene bg main_summer with Dissolve(1.0)
    show bg main summer:
        yalign 1.0
        easeout 0.5 yalign 14.7/15
        linear 15 yalign 0.05
        easein 1 yalign 0.0
    旁白 "{cps=20}他的第一部作品发表，却成为昙花一现。{w=1.2}{nw}"
    旁白 "{cps=20}而她由于工作压力和长久的劳累患上了失语症。{w=1.2}{nw}"
    旁白 "{cps=20}他决定带着她去旅行，同时用自己写的故事来重新刺激她的语言功能。{w=1.2}{nw}"
    show bg main winter with ImageDissolve("liluo_common/common/transition/left.png",
                                time = 1.5, ramplen = 256, reverse = False)
    旁白 "{cps=20}故事的最后，她的语言功能恢复了，二人在宁静的古都住下。{w=1.3}{nw}"
    旁白 "{cps=20}最后一幕中，初雪那一天清晨，二人共同欣赏日出。{w=1.3}{nw}"
    旁白 "{cps=20}空气凝滞冰清，而市街也还在沉睡着。{w=1.3}{nw}"
    scene white with tran_dissolve_up(3.0, ramplen = 1024)
    pause 1.0
    scene black with Dissolve(3.0)
    pause 1.0
    stop music
    
     
