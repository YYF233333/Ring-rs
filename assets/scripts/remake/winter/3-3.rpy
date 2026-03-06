label chapter14:
    #TODO:本章全部cg都需要冬景版本
    #第三幕，第三场。
    #TODO: 这张背景是夏天。。。赶紧整个冬景来
    scene bg daxuexiaoyuan_xiaoyuandaolu_a with Dissolve(1.2)
    旁白 "12月底"
    旁白 "一直以来我都在构思着校内选拔的剧本。"
    旁白 "我翻了很多书，也问过一些戏剧专业的同学。然而，越是前进，越是感觉陷入了无边无际的黑暗中。"
    旁白 "我甚至小范围调查了同学们的阅读喜好。为了避免过于个性化，只有写更多人喜欢看的故事，才更容易得到高评价。"
    ###几个镜头轮切，宿舍-主干道-图书馆##
    #TODO: 这里是要一组还是要多组
    #FIXME:yysy我是想着表现时间流逝的，如果可以多组那就多组。不过如果觉得太繁杂就不用。
    scene bg ziwendedaxuesushe_n with tran_rf()
    pause 1.0
    scene bg daxuexiaoyuan_xiaoyuandaolu_n with tran_df()
    pause 1.0
    scene bg daxuexiaoyuan_tushuguan_n with tran_black_anticlockwise()
    旁白 "但是，作业实在是太多了。而且临近期末考试，我不得不复习落下的内容。"
    旁白 "晚上11点，我仍然在图书馆补上周的作业。"
    子文 "算积分也太痛苦了。这都啥啊…"
    朝希 "哦，是子文啊，这么晚了还不回去吗？"
    子文 "再不交平时分没了啊…"
    朝希 "你在写什么啊？"
    子文 "为什么这么难算啊——我为什么要学数学啊——"
    朝希 "唔…"
    子文 "没事，你先回宿舍吧。我算完这一个题就回去。"
    朝希 "哦，好吧。本来想叫你一起去吃夜宵的。"
    子文 "哦，那你们先去吧。记得给我留一份啊。"
    朝希 "还有，明天不是周六吗，几个舍友打算去南湖骑车，期末考前最后去放松一次咯。你要来吗？"
    子文 "唔。"
    旁白 "骑车么…"
    旁白 "本来想继续写剧本的。不过南湖的景色还是很不错的，嘛，权当积累素材了。"
    子文 "好啊。"
    朝希 "哇，不得了了，子文愿意出门了。"
    子文 "你这是几个意思，我难道不出门吗？"
    朝希 "没有没有，就是子文平时总是趴在电脑前面写东西，每次团建总是缺你。这次竟然惠然肯来，我实在太感动了。"
    子文 "…行了行了，别烦我了，快去吃你的夜宵吧。"
    朝希 "好嘞，回见！"
    scene bg ziwendedaxuesushe_n with tran_black_anticlockwise()
    旁白 "..."
    旁白 "一旦开始执笔就容易忘记时间。"
    旁白 "这种时候我喜欢去外面转转。偶尔也能获得一些灵感。"
    scene bg daxuexiaoyuan_xiaoyuandaolu_nt with tran_lf()
    旁白 "湖城大学的校园是很漂亮的，我尤其喜欢这条主干道。这条路也是去图书馆的必经之路。"
    旁白 "只要站在这条路上，我就有了我如今已经身处湖城大校园中的实感，也有了如今拥有着眼前的一切的实感。"
    旁白 "红叶现在怎么样了呢...虽然大体顺利，但是可能会很辛苦吧。"
    旁白 "我也不得不前进了呢。"
    ###道路夜景##
    子文 "呼啊，好难。"
    旁白 "打了一个大哈欠。"
    旁白 "总感觉情节发展已经不再受我掌控了。越发地感到痛苦。"
    旁白 "写作到底有什么意义呢…"
    旁白 "我从包里拿出了湖城大玉樱赏征文的宣传海报。"
    scene bg jijiangxiayu with tran_dissolve_down(ramplen = 256)
    子文 "到底应该怎么写呢…"
    scene black with tran_close(1.2)
    pause 1.0
    ##场景切到南湖（注 "原型为巢湖）##"
    play music "audio/BGM/9.ローファイ少女は今日も寝不足.mp3"
    scene bg hubianbaitian with tran_anticlockwise()
    pause 1.0
    scene bg daolubaitian with tran_rf()
    朝希 "我说过吧，这里还是很漂亮的，这样在湖边骑车可谓是一种享受啊。"
    子文 "倒也确实是开阔思路的一个好地方…这个素材记下了。"
    朝希 "啥？"
    子文 "没啥。风景确实好啊。"
    朝希 "不过，呼——今天怎么这么冷。"
    子文 "谁叫你疯狂飙车，然后又把外套全脱了。你这样保准感冒，没几天就要考试了，当心点嗷。"
    朝希 "怎么可能。好怂一子文。"
    子文 "说起来，明年的玉樱赏征文，朝希你要参加吗。我看你平时不也挺喜欢写东西，之前还在文学社的刊物上看到了你的投稿——"
    朝希 "参加肯定会参加啊，校内学生又不要报名费，相当于免费拿一份参赛作品集汇的杂志。"
    朝希 "不过估计是很难拿到奖，被选入玉樱赏提名就更不可能了。我拿什么和那帮文科生比嘛。"
    子文 "确实啊。来湖城大之前我还不知道有这么多厉害的人，更想不到这些人会强的那么离谱…"
    朝希 "也不用那么当真吧。反正也就是写着玩玩的，自己开心就好了。"
    朝希 "行了，继续骑咯，全程快40公里呢，累的受不了就打我——电——话——啊——（骑走）"
    子文 "写着玩玩吗。"
    旁白 "我启程前进。"
    stop music
    scene black with tran_rf()

    ###夜景##
    scene bg daxuexiaoyuan_xiaoyuandaolu_n with dissolve
    play music "audio/BGM/1.ensolarado.mp3"
    子文 "唔，明天要我过去，这么急是有什么事情吗？"
    红叶 winter_scarf_backhand5 "想见到你，这个理由还不够急吗？"
    子文 "哦哦哦好好好，我全理解了，我现在就打车赶过去——"
    红叶 winter_scarf_normal3 "也不是那么急啊！主要是，明天我们的录音室有一个开放活动，会给志愿从事演艺活动的学生们参观。"
    红叶 "所以我也想让你看看我平时是怎么工作的。"
    子文 "可是我这煞风景的嗓音，一开口就要被认出来是混进来的吧？"
    红叶 winter_scarf_normal5 "嘿嘿，我有内部通行许可啊。总之你明天要来一趟。"
    #scene black with dissolve

    ###场景切换到红叶的录音室##
    scene bg luyinshi with tran_black_close()
    show hongye winter_scarf_normal2 at s11
    旁白 "红叶的录音室。"
    旁白 "红叶如之前在学校的录音室一样，紧紧攥着台词本，站在麦克风前。"
    子文 "帅啊红叶，这和你以前的气场又完全不一样了啊。"
    红叶 winter_scarf_backhand4 "嗯？那我以前是什么样子啊？"
    子文 "以前红叶会看起来更紧张一点，感觉非常的严肃。现在的红叶看起来很愉快很轻松啊。"
    红叶 winter_scarf_normal3 "你这是什么意思啊，就算我现在只是在做一个示例，并不是真的在工作，那也是非常严肃认真的好吧？"
    子文 "我是说，现在的红叶大小姐要更可爱更迷人哦。"
    红叶 winter_scarf_backhand3 "…我们的大作家子文同学，你什么时候能说点至少不那么土的话呢？"
    hide hongye with dissolve
    #scene black with dissolve
    scene bg shiwusuo with tran_with_black(tran_lf())
    show tianlan a at s11
    pause
    旁白 "天兰在四处参观。"
    天兰 b "还真是怀念啊…"
    hide tianlan with dissolve
    #scene black with dissolve
    ###红叶的演出##
    scene bg luyinshi with tran_with_black(tran_rf())
    show hongye winter_scarf_normal1 at s11
    子文 "有一点我有点奇怪啊…"
    红叶 winter_scarf_backhand4 "嗯？"
    子文 "就是如果配音的时候遇到一些比较尴尬的场景的话，会不会弄一个帘子遮住呢。"
    红叶 winter_scarf_normal3 "你在说些什么啊！你不会指望我配那种场景吧…那种事情我肯定做不到啊…"
    子文 "怎么说呢，那种场景总是要有人来演出的吧…"
    红叶 winter_scarf_backhand2 "我才不会去做那样的事情！"
    子文 "但是我确实非常好奇啊。"
    红叶 winter_scarf_normal6 "呼…如果是工作的话，那该怎么样就是怎么样咯，不会有什么帘子。啊啊啊，我的觉悟还是不够啊，明明在工作中不应该带有太多自己的情绪的…"
    子文 "原来都是有这样忘我的决心吗…"
    show hongye winter_scarf_normal4 at s22
    show tianlan at s21
    天兰 b "不完全是哦。"
    #TODO: 其实这句还是挺长的
    天兰 "虽然作为声优，首要的工作是去理解角色。但是这不意味着我就是要去完全抹去自己的存在。"
    天兰 "放下自己的情绪为的是更好地理解角色，而保留自己的“角色”是为了更好地演绎角色。毕竟，那是“我”去演绎的角色啊。"
    红叶 winter_scarf_backhand4 "天兰前辈！"
    天兰 g "好久不见。你是叫…红叶？红叶小姐这么年轻就在这样有名的事务所工作，真的是很厉害呢。"
    红叶 winter_scarf_normal1 "哪里…但是您为什么会在这里…"
    天兰 a "我也是“对演艺事业有兴趣”的人嘛。听说这里有开放参观，就过来了，没想到能碰到你们。"
    子文 "您是住在湖城吗？"
    天兰 a "其实我本就住在这里，只是为了事业才去了江城。自从我出院，我就回到了这里。"
    天兰 "啊，相比长江，我还是更喜欢南湖一点吧，感觉只要站在湖风里，整个人都会被治愈呢。"
    stop music 
    红叶 winter_scarf_backhand4 "…"

    #TODO: 这个效果不是很满意
    window hide
    show hongye at tran_blur
    show tianlan at tran_blur
    
    show bg luyinshi at tran_blur
    pause 0.75
    scene black with dissolve
    scene bg xiaocanguan_n at blured
    show tianlan b at s11(easein_time = 0)
    show tianlan at blured
    with dissolve
    show bg xiaocanguan_n at antiblur
    show tianlan at antiblur
    ##回放场景
    天兰 "我已经再也配不出比那更精彩的作品了。"
    window hide
    show tianlan at tran_blur
    show bg xiaocanguan_n at tran_blur
    pause 0.75
    scene black with dissolve

    scene bg shiwusuo with tran_open()
    window auto
    旁白 "…"
    show fengdao c at s11
    旁白 "丰岛亦在参观。"
    丰岛 "…"
    window hide
    show fengdao at s22
    show tianlan a at s21
    pause
    #旁白 "随后他看见了天兰。"
    window auto
    丰岛 a"啊？为什么…"
    show fengdao at leave_right
     
