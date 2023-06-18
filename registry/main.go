package main

import (
	"fmt"

	"github.com/CompeyDev/boxer/registry/constants"
	"github.com/CompeyDev/boxer/registry/routeManager"
	"github.com/CompeyDev/boxer/registry/utils/logger"
	"github.com/gin-gonic/gin"
	"github.com/gookit/color"

	"github.com/google/uuid"
)

func main() {
	// Initial setup
	constants.RegisterOrSet("instanceId", uuid.New().URN())

	tmpSrv := NewServer()
	tmpSrv.Instance.Use(gin.Recovery())
	tmpSrv.Instance.Use(logger.Middleware())

	constants.RegisterOrSet("server", tmpSrv)

	// Registration and execution
	srv := constants.Get("server").(Server)

	routeManager.PopulateSelf()
	RegisterRoutesToManager()
	routeManager.RegisterToServerInstance()

	logger.Info("  CORE  ",
		fmt.Sprintf(
			"API server instantiated and listening on port %s.",
			color.Bold.Sprint(
				color.Yellow.Sprint(constants.Get("port")),
			),
		),
	)

	srv.Run()
}
