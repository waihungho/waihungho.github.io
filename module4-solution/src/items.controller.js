(function () {
'use strict';

angular.module('Data')
.controller('ItemsController', ItemsController);

ItemsController.$inject = ['items'];
function ItemsController(items) {
  var itemCtrl = this;
  itemCtrl.name = items.data.category.name;
  itemCtrl.short_name = items.data.category.short_name;
  itemCtrl.special_instruction = items.data.category.special_instruction;
}

})();
