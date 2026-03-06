label chapter11:
    #第三幕第一场
    ###高铁站##
    play music "audio/BGM/14-ED1.野良猫は宇宙を目指した.mp3"
    scene bg huchengzhan with tran_anticlockwise()
    show hongye summer_normal3 at s11
    红叶 summer_normal3"快点快点！高铁停车时间很短的。"
    show hongye at hide_left
    子文 "啊啊啊你倒是给我背一个包啊。"
    旁白 "我被迫背着满身行李追上去。"
    scene black with dissolve
    
    ###车厢里##
    #TODO:车厢cg
    旁白 "我戴着新买的，属于自己的耳机，心满意足地听音乐。"
    show bg 长江 with tran_rf()
    红叶 "子文快看！过长江了！"
    子文 "ZZZ…"
    红叶 "喂！"
    子文 "哇你不要老是这样吓人啊。"
    
    ###江城北站##
    scene bg gaotiezhan_n with tran_black_anticlockwise()
    show hongye summer_normal4 at s11("close")
    红叶、子文 "啊拜托请让一下…让一下…不要挤啊。"
    show hongye at hide_left
    
    ###江城北站站前广场##
    旁白 "好容易挤出了站"
    show hongye summer_normal1 at s11
    红叶 summer_normal1 "要拍了哦。"
    子文 "所以说为什么要拍我…"
    旁白 "我正在吃出站时买的薯条。"
    红叶 summer_normal3 "不准抱怨，给我好好站好了。"
    hide hongye
    旁白 "这时一只鸟唰地飞过，反应过来时我手里的薯条已经飞了。"
    子文 "哇，还我薯条啊。"
    
    
    ###某名胜古迹（其实是黄鹤楼）##
    scene bg 长江 with tran_black_anticlockwise()
    旁白 "我们站在台阶上，面对着长江。"
    红叶 "啊，真是累死我了。这景区里面这么大吗？"
    子文 "我说了是因为我们走错了路啊，白白从外面绕了一大圈。"
    红叶 "诶？可是我不是开着导航吗？"
    子文 "…你看导航之前至少要搞清楚哪里是北吧？"
    
    scene bg guoanlujiejing_jiejing_ with tran_black_anticlockwise()
    pause
    scene bg manzhanxianchang_n with tran_with_black(dissolve)
    show hongye summer_normal4 at s22
    旁白 "国安路步行街，江城玉樱节现场。"
    子文 "我的天哪…合着这就是个巨型漫展啊。"
    show hongye summer_normal5 at s11
    红叶 summer_normal5 "玉樱节可不止漫展哦。每年六月，全国的新电影、广播剧CD、游戏、漫画以及小说都会在这里展出。是全国仅有的超级文艺盛会哦。"
    红叶 summer_normal1 "作者会来签售作品，导演和演员会来和观众见面，声优和剧本家也不例外。这也是新人借机打出名声的绝好机会啊。"
    子文 "那这岂不是太卷了…"
    红叶 summer_backhand4 "你这话说的，没有点真本事，你都过不了第一轮评选。{w=0.5}{nw}"
    show bg guoanlujiejing_dianpu_cu11 with tran_rf()
    extend "哇，一整个货柜的签名CD吗，哦是展览品啊… "
    
    子文 "我天，这家店里竟然有上周才发售的桜季的新作吗？哦原来是生肉啊，打扰了…"
    ###商店街，抓娃娃机##
    #TODO:有没有抓娃娃机的背景
    scene bg guoanlujiejing_dianpu_n with tran_with_black(tran_rf())
    旁白 "抓娃娃机。"
    show hongye summer_backhand3 at s21
    红叶 summer_backhand3"啊，又是只差一点。为什么这个抓手一定会松一下啊。"
    子文 "大概是这样设置的吧…毕竟不能让人轻易抓到娃娃。"
    hide hongye with dissolve
    旁白 "我投入两个硬币，尝试抓了一下。没想到这抓手竟然带着娃娃移动到了出口。我很轻松地就拿到了这个娃娃。"
    红叶 summer_backhand3 "你是怎么做到的？我不能接受！"
    子文 "我也就这样抓了一下啊…好吧好吧这个送给你咯…喂不要拧我的耳朵。"
    
    scene bg guoanlujiejing_dianpu_cu11 with tran_black_anticlockwise()
    show hongye summer_normal2 at s11
    stop music
    旁白 "CD店铺，红叶在查看今年的玉樱赏提名作品。"
    旁白 "太阳逐渐西斜。"
    旁白 "街边有一个抱着一把吉他演唱的女歌手。在玉樱赏期间，国安路附近会有不少这样的歌手。"
    旁白 "她的歌声非常好听，因此我们暂时驻足欣赏。"
    show hongye summer_normal4 at s22
    show tianlan a at s21
    旁白 "红叶看起来像是在努力回想什么。"#TODO:这句感觉可以直接换成。。。
    旁白 "曲罢，我鼓起掌，旁边也有路人停下来鼓掌，但是随即都离开了。"
    红叶 summer_backhand4 "我知道这首歌…"
    子文 "啊？"
    红叶 summer_backhand2 "天兰演唱的《致往昔》"
    show tianlan b at s21
    歌手 "诶，竟然有人还记得这首歌啊。"
    红叶 summer_normal1 "这是…这是好几年前，我初一那一年听到的一个玉樱赏提名的连载广播剧的片尾曲！"
    红叶 "不得不说，您唱的真的很好。这首歌很难，我也非常喜欢。您唱的已经堪比原唱了！"
    天兰 g "诶，谢谢夸奖。不过，我其实就是原唱呢…"
    红叶 summer_normal4 "啊？"
    天兰 a "其实，我也是那个广播剧的声优，然后演唱了作为ED的这首歌。这位小姐应该也记得吧。那个时候我还不知道，那就是我配音的最后一个有姓名的角色了呢。"
    红叶 summer_backhand4 "怎么会…前辈的唱功和声音明明都那么好…"
    天兰 e "我一直都有肺病。那个时候我住了三年院。等我再走出医院时，这个业界已经没有了我的位置了。"
    天兰 g "嘛，也没什么办法。我在录制这首歌的时候，还想着或许有一天可以染指玉樱。现在我只能在玉樱节时，抱着吉他在这里唱《致往昔》了呢…"
    红叶 summer_normal2 "为什么…"
    天兰 a "嗯？"
    红叶 summer_backhand2 "前辈为什么不重新试一试呢。重新为主角试音，再一次站在麦克风前。这样放弃，不是太可惜了吗？"
    旁白 "天兰摇摇头。"
    天兰 b "其实，是我觉得，我已经再也配不出比那更精彩的作品了。那是我此生仅有的机会了。"
    天兰 "而且，现在的生活其实也不错啊。虽然我演出的角色没有姓名，但是，他们都可以叫“天兰”。"
    天兰 a "也不早了，我也得回去了。不管怎么说，遇到了还记得我的歌的老粉，还是非常令人开心呢~"
    show tianlan at hide_left_with_transparent with Pause(0.5)
    红叶 summer_backhand6 "这样吗…"
    hide hongye with dissolve
    旁白 "……"
    

    ###房间##
    scene bg jiudianfangjian_moonlight with tran_close_open()
    show hongye pajamas_normal5 at s11
    play music "audio/BGM/4.終幕への間奏曲.mp3"
    红叶 pajamas_normal5 "啊，可累死我了。不过还是很开心啊。"
    子文 "今天确实走得挺远了。"
    红叶 pajamas_backhand1 "子文你明天能不能背着我走啊…我感觉我快要瘫掉了。"
    子文 "…还是坐下歇一歇吧。"
    子文 "说起来，玉樱赏颁奖是明天吗？"
    红叶 pajamas_normal5 "对，就在明天。"
    show hongye pajamas_normal1 at s22
    旁白 "红叶翻出了白天拍到的CD的封面和声优的相片。"
    show hongye pajamas_normal1 at s11
    红叶 pajamas_normal5 "明天，象征着声优界至高荣誉的玉樱赏将会颁给这几个前辈之一。啊，好羡慕他们有这么好听的声线和这么稳的气息啊。"
    子文 "红叶你以后也要参与比赛吗？"
    红叶 pajamas_backhand4 "当然了。说起来，子文你有把我们的广播剧投出去吗？"
    子文 "…没有。就算投了也会在第一轮就被刷下来吧。"
    红叶 pajamas_backhand6 "确实啊。那充其量也就是两个学生一晚上赶工的作品，怎么可能有竞争力呢。这样下去只怕连提名都拿不到啊。"
    子文 "未必吧。那毕竟是凝聚了我们心血的作品啊。"
    旁白 "红叶捂着脸。"
    红叶 pajamas_normal6 "有什么用呢…天才只需抬头就能精准地指出月亮的位置，迈步就能抵达38万公里以外的终点，随后无数的普通人开始前赴后继地铺路。"
    红叶 "纵使苦干一辈子，到头来连天才的起点都没能摸到，然后带着遗憾进了坟墓。"
    子文 "但是…"
    红叶 pajamas_normal4 "嗯？"
    子文 "如果作为一件雕塑，那么月亮的美不可超越。而作为故事的话，千万块不同的砖要比月亮更加有趣。"
    红叶 pajamas_backhand4 "诶？什么意思？"
    子文 "我会觉得与红叶大小姐共同在图书馆度过的一夜比任何广播剧都要精彩一百倍。"
    show hongye pajamas_backhand1 at s11
    旁白 "说完我才感觉到这话有多么羞耻。"
    红叶 pajamas_backhand4 "诶？"
    子文 "所以我会继续写下去，就算只是作为业余爱好。我已经想明白了。"
    红叶 pajamas_backhand5 "子文？"
    子文 "嗯？"
    红叶 pajamas_normal5 "该说不愧是你呢。"
    旁白 "红叶笑了。"
    子文 "我怎么了。"
    红叶 pajamas_normal1 "湖城大学每年都会固定有一个玉樱赏的提名哦。只要通过校内选拔就能登上这个舞台。"
    子文 "饶了我吧，毕业旅行还要谈志愿的事情吗。"
    红叶 pajamas_backhand5 "如果你继续写的话，"
    子文 "什么？"
    红叶 pajamas_backhand1 "我一定会为你配音。"
    show hongye pajamas_backhand5 at s11("close")
    show hongye at hide_left
    旁白 "她突然凑到我耳边猛吹了一口气。"
    子文 "哇你这人…"
    stop music
    
    scene bg guoanluhuichang_hall_n with tran_black_anticlockwise()
    旁白 "次日，国安路主会场。江城玉樱赏颁奖仪式现场。"
    子文 "哇，好强的威压感。"
    旁白 "红叶面色凝重，紧紧握着拳头，目不转睛地盯着台上。"
    scene bg guoanluhuichang_stage_n with dissolve
    show fengdao b at s11("far")
    丰岛 b "我们相信，好的作品会改变一个时代。于是这个时代也因为这些作品而变得伟大。如今它们正齐聚一堂，以“创作”的无上力量，宣示着什么是“伟大”…"
    hide fengdao 
    scene bg guoanluhuichang_hall_n with dissolve
    旁白 "我深吸一口气。"
    子文 "这个主持不是一般人吧…"
    红叶 summer_normal3 "…"
    子文 "看来你真的很向往那个地方呢。"
    scene bg guoanluhuichang_hall_n with tran_with_black(tran_lf())
    旁白 "一个来晚了的湖城某二流娱乐小报记者走进了会场。"
    旁白 "他来回转了几圈，由于前排已经全部被大报社的记者占了位置，他只能悻悻地走到侧面的观众席。"
    记者 "「嗯？落英？」"
    
    #TODO:这一段也许需要插入BGM但是我没想好@wzy
    scene bg guoanluhuichang_hall_ with tran_black_anticlockwise()
    旁白 "…"
    旁白 "颁奖结束了，大家四散离场。我和红叶被人潮冲散。我被挤到一边，于是逆着人潮掉头回去寻找红叶。"
    show hongye summer_backhand3 at s11("far")
    旁白 "红叶停在原地周围全都是人。"
    红叶 summer_backhand3"…"
    记者 "…"
    show hongye summer_normal2
    记者 "您是落英小姐吧。您刚刚获得了湖城玉樱赏的新人赏，现在却出现在江城玉樱节的现场，是有什么打算呢？"
    红叶 summer_normal4 "诶？不是…"
    记者 "您获得了湖城玉樱赏，但是连江城玉樱赏的提名都没有获得，您如何看待这件事呢…"
    红叶 summer_backhand2 "不，不是这样的…"
    记者 "落英小姐，前些时间有传闻，说您和负责您节目的配音导演来往密切，是有这件事吧？"
    红叶 summer_normal4 "你…"
    show hongye at hide_right
    #TODO:这里可以放一堆炒饭(doge)
    #TODO:这段有个巨大演出要做
    旁白 "周围有认识“落英”这个名字的人，已经注意到了骚乱，迅速聚拢过来，把红叶围在了中央。"
    image 看客A = "boy.png"
    image 看客B = "girl.png"
    image 看客C = "boy.png"
    image 看客D = "girl.png"
    image 看客E = "boy.png"
    image 看客F = "girl.png"
    image 看客G = "boy.png"
    image 看客H = "girl.png"
    image 看客I = "boy.png"
    image 看客J = "girl.png"
    show 看客A at s42("far")
    看客A "落英？她为什么会在这里？"
    show 看客B at s22
    看客B "估计是没有获得提名，嫉妒了吧。"
    show 看客C at s31("close")
    看客C "原来落英就这啊…"
    window hide
    show 看客D at s33("close")
    pause 0.4
    show 看客E at s11("far")
    pause 0.4
    show 看客F at s11("close")
    pause 0.4
    show 看客G at s21("close")
    pause 0.4
    show 看客H at s21("far")
    pause 0.4
    show 看客I at s33("far")
    pause 0.4
    show 看客J at s22("close")
    window auto
    旁白 "红叶被前推后搡，被完全淹没在了人群中。"
    子文 "…那群人在干什么？"
    子文 "红叶？！"
    旁白 "我立刻挤过去。"
    子文 "请让一下…"
    hide hongye
    show hongye summer_normal6 at s11("far")
    旁白 "红叶手足无措。"
    子文 "该死的…"
    旁白 "几个看热闹的大块头把路堵的死死的。我简直像一只蚊子。"
    旁白 "我从他们的间隙中强行穿过去。这时我才注意到骚乱的始作俑者，那个满脸猥琐的记者。"
    旁白 "我一把把拍摄的PV机抢了过来。"
    记者 "喂，你搞什么？"
    旁白 "确认了没有能够保存录像文件，我直接把这个PV机丢到了人堆外围。"
    红叶 summer_normal4 "子文…"
    记者 "你他妈…"
    子文 "你认错人了，这位是红叶，我的未婚妻。"
    记者 "你他妈怎么抢老子相机？"
    子文 "都说了你认错人了，赶快滚，不然…"
    旁白 "记者扯住我，我反手锁住他的胳膊。"
    子文 "我不认识什么落英。现在你敢动她一根指头，老子把你胳膊扭断！"
    旁白 "我加大了力度。记者被半按倒，因疼痛而痛苦呻吟。"
    show hongye summer_normal4 at s11("close")
    show hongye at hide_left
    旁白 "保安已经赶过来，外围的人开始变散。我拽着红叶，飞也似地冲出了人群，直接跑出了会场。"
    scene black with dissolve
    旁白 "……{w=0.5}{nw}"
    scene bg guoanlujiejing_jiejing_y with dissolve
    pause 1.0
    scene bg jiangchengjiejing_y with tran_rf()
    pause 0.5
    红叶 summer_backhand4 "停——一——下——啊——"
    旁白 "我拉着红叶跑过了一段铁道。我确认了周围没有人追过来，于是停下来。"
    show hongye summer_normal4 at s11
    红叶 "呼啊…呼啊…呼——我的天，你搞什么，快把我整死了。我可是穿着裙子啊？！"
    子文 "哈——哈啊，我脑子一热，就这样拉着你跑出来了。"
    红叶 summer_backhand1 "声优可是有在每天锻炼的哦。你看看你，到底怎么有的胆量这样…"
    子文 "红叶，刚刚我想明白了。"
    play music "audio/BGM/15.夕陽～君に幸あれ～.mp3"
    旁白 "我深呼吸一口。"
    #红叶 summer_normal1 "什么？"
    scene cg1_2 with tran_with_black(Dissolve(1.0))
    子文 "{cps=15}我从很久很久以前就在写——{w=0.5}写一个故事，"
    子文 "{cps=15}写了夏天"
    子文 "{cps=15}写了暑风"
    子文 "{cps=15}流云"
    子文 "{cps=15}大海"
    子文 "{cps=15}远山"
    子文 "{cps=15}夕阳"
    子文 "{cps=15}野草"
    子文 "{cps=15}铁道"
    子文 "{cps=15}许许多多的东西"
    子文 "{cps=15}但是还没有写完。"
    旁白 "二人面对站立。"
    子文 "{cps=15}直到我写下："
    子文 "{cps=15}我——喜欢你。"
    红叶 "那么这是故事的结局吗？"
    子文 "这同样不是结局。我意识到的时候，我已经在这里了。"
    "一阵风吹过，带起了一些花瓣在空中飞舞。"
    show cg1_1 with dissolve
    红叶 "你还真是完全没有演艺天赋呢…"
    子文 "是啊，我恐怕这辈子也不可能成为一个声优的吧。"
    子文 "但是，我无论如何都喜欢你，比任何其他人都要爱着你。"
    子文 "所以说，请你，和我一起，见证这部即兴剧的下一幕吧。"
    scene black with dissolve
    stop music 
    #…
    #（！！！！！！这里我需要CG，我需要CG，我需要CG！！！！！！）
    #…
    #TODO:场景切换到回程的高铁。透过车窗可以看见，二人的手紧紧地握在一起。
     
