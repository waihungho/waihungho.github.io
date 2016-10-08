(function () {
'use strict';

angular.module('Data')
.controller('MenuAppController', MenuAppController);

MenuAppController.$inject = ['items'];
function MenuAppController(items) {
  var menuCtrl = this;
  menuCtrl.items = items.data;
}

})();
