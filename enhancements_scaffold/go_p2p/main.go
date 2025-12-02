package main

import (
    "bufio"
    "context"
    "fmt"
    "io"
    "log"
    "os"
    "os/signal"
    "syscall"
    "time"

    libp2p "github.com/libp2p/go-libp2p"
    pubsub "github.com/libp2p/go-libp2p-pubsub"
    peer "github.com/libp2p/go-libp2p/core/peer"
    host "github.com/libp2p/go-libp2p/core/host"
    network "github.com/libp2p/go-libp2p/core/network"
    ma "github.com/multiformats/go-multiaddr"
)

const EchoProtocol = "/neonet/echo/1.0.0"
const TopicName = "neonet-topic-1"

func setupHost(ctx context.Context) (host.Host, error) {
    h, err := libp2p.New()
    if err != nil {
        return nil, err
    }
    return h, nil
}

func echoHandler(s network.Stream) {
    defer s.Close()
    r := bufio.NewReader(s)
    for {
        line, err := r.ReadString('\n')
        if err != nil {
            if err != io.EOF {
                log.Println("echo handler read error:", err)
            }
            return
        }
        // echo back
        _, _ = s.Write([]byte("echo: " + line))
    }
}

func main() {
    ctx, cancel := context.WithCancel(context.Background())
    defer cancel()

    h, err := setupHost(ctx)
    if err != nil {
        log.Fatal(err)
    }

    h.SetStreamHandler(EchoProtocol, func(s network.Stream) {
        log.Println("New echo stream from", s.Conn().RemotePeer())
        echoHandler(s)
    })

    // create pubsub
    ps, err := pubsub.NewGossipSub(ctx, h)
    if err != nil {
        log.Println("failed to create pubsub:", err)
    } else {
        topic, err := ps.Join(TopicName)
        if err == nil {
            // start a subscriber
            sub, err := topic.Subscribe()
            if err == nil {
                go func() {
                    for {
                        msg, err := sub.Next(ctx)
                        if err != nil {
                            return
                        }
                        log.Printf("pubsub: msg from %s: %s\n", msg.ReceivedFrom.Pretty(), string(msg.Data))
                    }
                }()
            }
            // publish a welcome message
            _ = topic.Publish(ctx, []byte("node "+h.ID().Pretty()+" joined at "+time.Now().Format(time.RFC3339)))
        }
    }

    // print listen addresses
    addrs := h.Addrs()
    for _, a := range addrs {
        fmt.Printf("Listening on: %s/p2p/%s\n", a.String(), h.ID().Pretty())
    }

    // simple CLI to dial a peer or open an echo stream
    go func() {
        in := bufio.NewReader(os.Stdin)
        for {
            fmt.Print("> ")
            text, _ := in.ReadString('\n')
            if len(text) == 0 {
                continue
            }
            text = text[:len(text)-1]
            if text == "quit" || text == "exit" {
                cancel()
                return
            }
            if text == "" {
                continue
            }
            // dial multiaddr like /ip4/127.0.0.1/tcp/4001/p2p/Qm...
            if text[0] == '/' {
                maddr, err := ma.NewMultiaddr(text)
                if err != nil {
                    fmt.Println("invalid multiaddr:", err)
                    continue
                }
                pi, err := peer.AddrInfoFromP2pAddr(maddr)
                if err != nil {
                    fmt.Println("addrinfo error:", err)
                    continue
                }
                if err := h.Connect(ctx, *pi); err != nil {
                    fmt.Println("connect error:", err)
                    continue
                }
                fmt.Println("connected to", pi.ID.Pretty())
                continue
            }
            // otherwise, expect "peerID message" to open stream
            var pidStr, msg string
            n, _ := fmt.Sscanf(text, "%s %s", &pidStr, &msg)
            if n >= 1 {
                pid, err := peer.Decode(pidStr)
                if err != nil {
                    fmt.Println("invalid peer id:", err)
                    continue
                }
                s, err := h.NewStream(ctx, pid, EchoProtocol)
                if err != nil {
                    fmt.Println("stream open error:", err)
                    continue
                }
                _, _ = s.Write([]byte(msg + "\n"))
                buf := make([]byte, 1024)
                n, _ := s.Read(buf)
                fmt.Println("reply:", string(buf[:n]))
                s.Close()
            }
        }
    }()

    // wait for interrupt
    ch := make(chan os.Signal, 1)
    signal.Notify(ch, syscall.SIGINT, syscall.SIGTERM)
    <-ch
    log.Println("shutting down...")
    _ = h.Close()
}
