(function () {
"use strict";

angular.module('public')
.controller('MyinfoController', MyinfoController);

MyinfoController.$inject = ['MenuService'];
function MyinfoController( MenuService) {
  var reg = this;
  reg.favorite=MenuService.getFavoritedish();
}

})();
