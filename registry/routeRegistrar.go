package main

import (
	"github.com/CompeyDev/boxer/registry/constants"
	"github.com/CompeyDev/boxer/registry/routes/heartbeat"
	types "github.com/CompeyDev/boxer/registry/utils/types"
)

func RegisterRoutesToManager() {
	heartbeat.Register(constants.Get("routeManagerStruct").(types.TRouteManager))
}
