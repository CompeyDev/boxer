package heartbeat

import (
	types "github.com/CompeyDev/boxer/registry/utils/types"
	"github.com/gin-gonic/gin"
)

func Register(routeManager types.TRouteManager) {
	routeManager.AddRoute("/heartbeat::GET", func(ctx *gin.Context) {
		ctx.JSON(200, gin.H{
			"status": 200,
		})
	})
}
