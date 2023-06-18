package routeManager

import (
	"fmt"
	"strings"

	"github.com/CompeyDev/boxer/registry/constants"
	"github.com/CompeyDev/boxer/registry/utils/logger"
	types "github.com/CompeyDev/boxer/registry/utils/types"
	"github.com/gin-gonic/gin"
)

var RoutesCollection = map[string]func(*gin.Context){}

func AddRoute(routePathAndMethod string, handler func(*gin.Context)) {
	RoutesCollection[routePathAndMethod] = handler
}

func PopulateSelf() {
	constants.RegisterOrSet("routeManagerStruct", types.TRouteManager{
		RoutesCollection: RoutesCollection,
		AddRoute:         AddRoute,
		PopulateSelf:     PopulateSelf,
	})
}

func RegisterToServerInstance() {
	srv := constants.Get("server").(types.TServer)

	for routePathAndMethod, handler := range RoutesCollection {
		s := strings.Split(routePathAndMethod, "::")
		path := s[0]
		method := s[1]

		logger.Info("  CORE  ", fmt.Sprintf("Registering route %s for method %s to server instance", path, method))

		srv.Register(method, path, handler)
	}

	constants.RegisterOrSet("server", srv)
}
