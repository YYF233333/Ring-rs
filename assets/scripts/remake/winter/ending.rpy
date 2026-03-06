label ending:
    #终幕
    ###连续的场景切换## 
    play music "audio/BGM/18-ED2.10℃.mp3"
    scene bg yiyuanwaijing_bleak with Dissolve(1.5)
    旁白 "我向红叶的父母告别，前往会场"
    ###会场##
    scene bg huchenghuichang_stair_n with tran_with_black(tran_rf())
    子文 "请——等——一——下——还有作品没有提交！！！"
    工作人员 "啊，我们已经截止收稿了…"
    show fengdao a at s11
    丰岛 a "稍等，这份稿件我收了。"
    子文 "先生，非常感谢…"
    丰岛 c "不用谢我。反倒是我应该谢谢你，为我展示了另一种结局。"
    子文 "啊？"
    hide fengdao
    scene bg huchenghuichang_stage_n with tran_close_open()
    ###主会场##
    旁白 "会场内熙熙攘攘。我由于来晚了，没有座位，于是只能站在过道里。"
    旁白 "我的头接触墙的时候，差点睡着。直到主持开口把我惊醒。"
    丰岛 a "今年湖城获得江城玉樱赏提名资格的——"
    旁白 "…"
    丰岛 b "《音与言语的即兴剧》，{w=1.0}{nw}"
    extend c "来自…啊哦，这个作者在上交稿件的时候怎么没有写名字啊？这位作者你在会场吗？"
    旁白 "…"
    旁白 "……"
    #GE
    ###医院##
    scene bg yiyuan_bingfang_y with tran_black_anticlockwise()
    旁白 "次日，病房内。"
    show hongye pajamas_normal1 at s11
    旁白 "红叶正在吃我刚买来的蜂蜜蛋糕。手术仅仅过了一天，她的胃口就已经恢复了。"
    旁白 "我难以掩饰欣喜的笑容。"
    红叶 pajamas_normal3 "我说，子文啊…"
    子文 "啊…什么？"
    红叶 pajamas_backhand3 "你从刚才开始就坐在那边傻笑，到底在笑些啥？"
    子文 "…我那叫傻笑吗？"
    红叶 pajamas_normal1 "是啊，你的表情从来藏不住。"
    子文 "我想到高兴的事情。"
    红叶 pajamas_normal4 "嗯？啥事啊这么开心？"
    子文 "没啥事，就是开心。"
    show hongye pajamas_backhand1 at leap
    红叶 "你过来。"
    子文 "…我不。"
    show hongye pajamas_backhand3 at leap("small")
    pause 0.28
    show hongye at leap("small")
    红叶 "耳朵伸过来一下。哎呀你怕什么，我又不会吃了你。"
    scene cg2_1 with dissolve
    旁白 "我靠近过去。红叶再次给了我一个膝枕。"
    子文 "噢噢噢噢噢你干啥？"
    红叶 pajamas_normal1 "嗯？和之前那次的比，不舒服吗？"
    子文 "舒服啊，很舒服啊。难道有人能拒绝红叶的膝枕吗？"
    红叶 pajamas_backhand3 "…你重新组织一下语言。"
    子文 "…难道我能拒绝我可爱的恋人的膝枕吗？"
    show cg2_2 with dissolve
    红叶 pajamas_normal5 "这才对嘛"
    旁白 "闭上眼睛。红叶轻轻地抚摸着我的头发。"
    红叶 pajamas_normal1 "需要一个今日限定膝枕ASMR吗？"
    子文 "别了吧，太羞耻了。"
    红叶 pajamas_backhand5 "这里也没有其他人哦。"
    子文 "…不要。"
    红叶 pajamas_backhand3 "什么嘛，其实你很想要吧。"
    旁白 "我没有回答。红叶的声音柔和起来。"
    hide cg2_2
    show cg2_1
    with dissolve
    红叶 pajamas_normal4 "所以你是要转去中文系吗？"
    子文 "是的。我想，有了这篇作品，学校不会拒绝我。"
    红叶 pajamas_backhand1 "你以后要继续写下去吗？"
    子文 "那是当然了。唔，这么一说，就算不转也会继续写下去啊。因为这个故事，无论如何都只会是属于我们二人的即兴剧。"
    旁白 "我坐起来。"
    子文 "说起来，你之前告诉我，是有三个愿望是吧？现在已经告诉了我两个，并且也都…算是实现了。那么第三个愿望是什么呢？"
    hide cg2_1
    show cg2_2
    with dissolve
    红叶 pajamas_normal5 "嗯…不告诉你。"
    子文 "喂，哪有这样许愿的啊？"
    hide cg2_2
    show cg2_1
    with dissolve
    红叶 pajamas_backhand3 "现在就是不告诉你。以后你会知道的。"
    旁白 "红叶温柔地看着我，她的侧脸被夕阳照亮，我不禁看入迷了。"
    #TODO: 动效修改
    #FIXME:yysy我甚至感觉这里是不是可以弄个滤镜啥的
    scene bg huanghun with tran_lf()
    pause 1.0
    ##演出 "夕阳##"
    
    ###图书室##
    scene bg fengdaodebangongshi_ with tran_with_black(Dissolve(0.5))
    旁白 "…"
    show fengdao a at s21
    旁白 "杂乱的档案室。丰岛正在收拾这次各地的入选稿件。"
    丰岛 e "我的天啊，这群人交作品也不知道备份一下，所有城市的稿件全都混在一起，我真的是吐了…"
    show fengdao a at s22
    show tianlan b at s21
    旁白 "天兰把一个文件夹丢给他。"
    天兰 "喏，这个。我收稿的时候整理的。"
    丰岛 c "还真的不愧是你啊，天兰。"
    天兰 a "呵，你还是老样子，一心扑在作品上，但是对这些东西总是不上心。"
    丰岛 c "不得不说，如果有你在的话，效率要高不少。"
    天兰 g "…稍等，容我撤回前言。"
    丰岛 a "啊？"
    天兰 b "你其实有一点点变了。"
    丰岛 a "嗯？你说哪里？"
    天兰 b "之前，那个女孩子问过我一个问题。"
    
    window hide
    show bg fengdaodebangongshi_ at tran_blur
    show tianlan at tran_blur 
    show fengdao at tran_blur
    pause .5
    scene black with dissolve
    scene bg xiaocanguan_n at blured
    ###场景切回女子会当晚的最后##
    show hongye summer_backhand4 at s22(easein_time = 0)
    show hongye at blured
    with dissolve
    show bg xiaocanguan_n at antiblur
    show hongye summer_backhand4 at antiblur
    window auto

    红叶 "天兰前辈，我有一个问题…"
    天兰 a "嗯？"
    红叶 summer_normal4 "您刚才说美是完全不可触及的东西…但是我分明感受到了，在故事中有一种类似旋律的东西，就像是故事的氛围之类的东西，直击我的内心。"
    红叶 summer_backhand2 "爱与美，为什么一定是对立的东西，也许有相当大的部分，是重叠的呢…在一个广播剧中，旋律与言语，到底哪个更重要一点呢？"
    天兰 d "…嗯…"

    window hide
    show bg xiaocanguan_n at tran_blur
    show hongye at tran_blur
    pause .5
    scene black with dissolve
    window auto
    
    ###切回档案室##
    scene bg fengdaodebangongshi_
    show tianlan b at s21(easein_time = 0)
    show fengdao c at s22(easein_time = 0)
    with tran_open()
    天兰 b "对于你而言，丰岛，你也许，也对这个问题感到过疑惑，然后现在已经有了你的答案了吧。"
    丰岛 c "…"
    旁白 "丰岛笑起来。"
    丰岛 b "Watermelon的电气石，音与言语的交叉点"
    天兰 b "色彩满溢的世界之中，七朵樱花飘舞而过，而我将向其追逐。"
    scene black with dissolve
    ###镜头给到雪化之后的世界##
    #CG
    scene cg3_1 with Dissolve(1.5)
    红叶 "下一幕会是什么样子呢？是会满溢着悲伤，还是会闪烁着幸福呢？"
    子文 "谁知道呢。但是有一点是确定的，那就是——"
    scene white with tran_dissolve_up(1.3, ramplen = 1024)
    #最后一句居中
    window hide
    image last = Text("{color=#000}{size=50}我们正是这部即兴剧的创作者。{/color}")
    show last at truecenter with tran_lf(1.7)
    pause
    hide last with tran_lf()
    window auto
    $ renpy.movie_cutscene("audio/ending_HVC_bgm.webm")
    stop music
    #FIN
     
