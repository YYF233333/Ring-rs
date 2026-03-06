label chapter5:
    #第一幕第五场
    ###子文的公寓内景##
    play music "audio/BGM/7.街を歩けば.mp3"
    scene bg ziwendegongyu_y with dissolve
    #TODO:两个房间的拼接@pipixia
    #FIXME:这个得ps干活
    红叶 pajamas_normal6 "抱歉，是我考虑不周了。"
    子文 "没有没有，这个完全是我自作自受。嗐，也不止这一次了，我哪次成绩稳定过嘛。"
    红叶 pajamas_backhand1 "我记得，初三那会，临近中考的时候，你不是拼了命地在学习吗，最后几次考试成绩都高的吓人，老师都说不像你了。"
    子文 "那都什么时候了，和现在能一样吗。再说，我那会还不是在想，总不能输给天天一起上学放学的家伙吧。"
    红叶 pajamas_normal5 "嘿嘿…"
    旁白 "红叶狡黠地笑了。"
    红叶 pajamas_backhand2 "我是觉得你应该听你爸的。"
    子文 "可是我也不知道我应该去学什么啊啊啊。写作可不是什么容易的东西啊，何况在现在这个时代，这么卷的环境，要决心靠写东西挣钱也太难了吧。"
    红叶 pajamas_normal2 "我之前说什么来着。"
    子文 "嗯？"
    红叶 pajamas_backhand3 "啊呀，这就忘了吗，看来我对于子文确实无足轻重呢。"
    子文 "不敢，红叶大小姐的教诲我谨记于心。只是，我还是时时地感觉到，我没有那样的才能，连自娱自乐都很困难啊——"
    红叶 pajamas_normal2 "唔，那倒是没有什么。但是至少，你可以再试一下。"
    子文 "试？"
    红叶 pajamas_backhand1 "至少，把这一次的剧本写完吧。哪怕你心里还残存着任何一点创作的火种，不妨现在就把它全部燃烧起来吧。"
    子文 "唔…"
    旁白 "我盯着眼前的作文纸。思绪回到了五年前。"
    stop music
    ###初中教室##
    play music "audio/BGM/8.紫苑_-追憶-.mp3"
    scene bg chuzhongjiaoshi_hi with pixellate
    pause 0.5
    camera:
        easein 0.5 zoom 1.1 xalign 0 yalign 0.5
    子文 "请读一下，这是我写的小说…"
    男生A "抱歉，我作业还没写完…"
    子文 "…"
    camera:
        linear 1.0 xalign 0.5 zoom 1.25
    pause 2.0
    子文 "有没有兴趣读一下，这是我新写的小说，不会花费很久的…"
    男生B "唔，我看看，诶，这是写什么的啊？"
    子文 "是写一个福利机构的公务员回家过年，然后碰到了一个可怜的乞丐…"
    男生B "没有恋爱元素吗…"
    子文 "抱歉，因为我也没有这样的经历，因此不是很擅长写这类…"
    男生B "你要是写恋爱小说，会更受欢迎一点的。"
    子文 "好，好的，谢谢。"
    #"子文转身离开之后听到了男生C的议论："
    camera:
        linear 1.5 xalign 1.0 zoom 1.4
    pause 1.0
    男生B "啥玩意啊就拿来给别人看，写的什么垃圾还要浪费别人的时间。"
    子文 "…"
    camera:
        easein 1.0 zoom 1.5
    pause 1.0
    show 女生A at s22("far")
    子文 "打扰一下，请…"
    #女生A "..."
    show 女生A at leave_right
    #旁白 "看也没看就离开了。"
    pause 1.0
    女生B "离那个家伙远一点。他们说这个人是怪物，成天喜欢弄一些奇奇怪怪的想法和故事。千万不要看他写的东西。"
    子文 "..."
    旁白 "我咬咬嘴唇。"
    stop music
    camera:
        linear 1.0 zoom 1.0
    pause 1.0
    show hongye summer_backhand1 at s22
    红叶 summer_backhand1 "嗯？这是什么。翻翻看吧。"
    show hongye summer_backhand4 at leap(degree = "small", easein_time = 0.08, easeout_time = 0.08)
    子文 "喂！"
    play music "audio/BGM/7.街を歩けば.mp3" 
    show hongye summer_normal4:
        easein 1.0 xalign 0.5
    红叶 summer_normal4 "哇，你吓死我了，能不能好好打招呼啊。"
    子文 "你怎么在看我的东西啊。"
    红叶 summer_backhand5 "唔，我心想是子文，也不会怎么在意…"
    子文 "…"
    旁白 "我把作文纸抢回来。"
    红叶 summer_backhand4 "啊，其实我倒是觉得写得挺有意思的。"
    子文 "你没有听他们说，看我写的东西会被诅咒吗，就像被脏水泼了一身一样。"
    红叶 summer_normal4 "…有这种事？"
    旁白 "红叶摇摇头。"
    子文 "都是很无聊的东西哦，没有王道故事没有热血格斗，没有校园恋爱，你真的要浪费时间来看吗，我的红叶大小姐？"
    红叶 summer_normal2 "我还是想看。"
    子文 "那好吧。"
    旁白 "我递出了作文纸。坦诚地说，我不抱什么希望了。"
    红叶 summer_normal4 "“一个做什么事情都像在梦游的人当上了警察”。这是个什么设定啊？"
    子文 "那不是梦游啊，只是漫不经心，始终感觉置身事外。"
    红叶 summer_backhand4 "那这样的人当上了警察，岂不是罪犯都要随便逃脱了？"
    子文 "那可不一定完全是坏事。反倒是这个人，他更容易看出执法司法系统中的问题。"
    红叶 summer_normal1 "这个意思啊…"
    hide hongye with dissolve
    ###窗外，从早上到下午，然后自然地切换到初三。##
    #TODO: 如果这里场景转换能带点模糊最好
    scene bg chuzhongjiaoshi_yu with Dissolve(2.3)
    show hongye winter_backhand4 at s11
    红叶 winter_backhand4 "“一度分别的幼驯染，偶然地重逢之后慢慢成为了恋人”，子文，你竟然开始写恋爱小说了？"
    子文 "小声点小声点，虽然我仍然没有经验，但是在初中这三年，总归还是见过一些事例的嘛。"
    红叶 winter_normal4 "诶，怎么到这里就没了啊，他们后来怎么样了啊？"
    子文 "我也没想好啊，只是想到这里，就写了这么多。"
    红叶 winter_normal1 "快写！以后我就是催更人咯。"
    子文 "喂，这催不得的吧。啊，果然这种题材要更受欢迎呢…"
    女生A "他们两个关系真好啊…"
    女生B "可不是吗，两个怪人，一个天天写千奇百怪的东西，一个喜欢用各种声音吓唬人。"
    $ music_class = channel_class()
    $ music_class.time = 0
    $ fading_pause(fade = 0.0, channel_class = music_class, filename = "audio/BGM/7.街を歩けば.mp3")
    红叶 winter_backhand3 "呵。在聊什么呢？"
    旁白 "是我完全认不出的声音。简直不像是红叶自己说出来的。"
    子文 "红叶，你那是？"
    $ fading_pause(start_adjust = -0.5, fade = 1.0, pause = None, channel_class = music_class, filename = "audio/BGM/7.街を歩けば.mp3")
    红叶 winter_backhand1 "改变声线而已。偶然的机会我发现我好像可以很容易模仿各种声音，于是他们说我是被别人附身了，其实都是我自己。"
    红叶 "但是这个事情就传开了，后来我也懒得争辩什么，所幸冷不防用奇怪的声音吓唬他们一下。"
    红叶 winter_normal2 "嘛，这倒也没什么，作为声优，这都是基本功了。"
    子文 "声优？"
    红叶 winter_backhand1 "就是配音演员。我想当声优，现在已经在练习基本功了。"
    子文 "这可是不得了的本事啊。"
    红叶 winter_normal1"这有什么特别的，反倒是大作家子文，这样的才华才是让我羡慕的呢。"
    子文 "我可不是什么大作家。相反，说到底，我写了这么多东西，也只有你乐意看啊。不过是不务正业搞出来的自娱自乐的玩意，拿不上台面，也永远不可能有什么成就的。"
    红叶 winter_backhand2 "为什么要这么想啊？难道你创作的动力都是假的吗？"
    红叶 "明明没有稿酬也要继续写下去，没有许多的读者也想精进笔力。这些难道都是装出来的吗？"
    子文 "…可是这些都是不切实际的…"
    红叶 winter_backhand4 "不切实际？{w=0.5}{nw}"
    show hongye winter_backhand2
    extend "不如说，硬要把一份义无反顾的热爱绑在一个什么东西上面，难道不是一种亵渎吗？"
    子文 "…不愧是红叶啊…"
    红叶 winter_backhand4 "诶？我说了什么吗？"
    子文 "这句话放在我新作的结尾可太合适了啊，好帅啊！"
    红叶 winter_normal3 "……所以说快去更新啊！"
    hide hongye
    scene bg ziwendegongyu_y with pixellate
    stop music
    旁白 "…"
    旁白 "我拿起了钢笔。"
    scene bg yekong with Dissolve(1.5)
    旁白 "倘若是写卷子，一般使用水笔。唯独创作的时候我才会使用钢笔。"
    play sound "audio/SE/33.翻书页（钢笔写字）.wav" volume 0.3
    pause .5
    play sound ["audio/SE/clock.mp3", "<silence .2>"] volume 2 loop
    旁白 "窗外是完全的死寂，耳边唯余秒针走动的清脆声响，仿佛是故事的节拍。"
    旁白 "我写下了标题："
    stop sound
    scene black with dissolve
    window hide
    play sound "audio/SE/writing-signature-1(硬笔).wav" volume 1.2 noloop
    image title = Text("{cps=13}{color=#ffffff}{size=50}《音与言语的即兴剧》{/color}{/cps}")
    show title at truecenter with tran_lf()
    pause 1.0
    hide title with tran_lf()
    pause 0.5
    window auto
     
