package main

import (
	"github.com/CompeyDev/boxer/registry/constants"
	"github.com/CompeyDev/boxer/registry/routes/heartbeat"
	"github.com/CompeyDev/boxer/registry/routes/meta"
	types "github.com/CompeyDev/boxer/registry/utils/types"
)

func RegisterRoutesToManager() {
	heartbeat.Register(constants.Get("routeManagerStruct").(types.TRouteManager))
	meta.RegisterFetcher(constants.Get("routeManagerStruct").(types.TRouteManager))
	meta.RegisterSetter(constants.Get("routeManagerStruct").(types.TRouteManager))
}
