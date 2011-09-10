USING: tools.deploy.config ;
H{
    { deploy-name "magog" }
    { deploy-ui? f }
    { deploy-c-types? f }
    { deploy-unicode? f }
    { deploy-console? f }
    { "stop-after-last-window?" t }
    { deploy-io 2 }
    { deploy-reflection 1 }
    { deploy-word-props? f }
    { deploy-math? t }
    { deploy-threads? t }

    ! Set to t for Factor's GUI widgets to work.
    { deploy-word-defs? f }
}
