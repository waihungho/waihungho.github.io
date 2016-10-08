(function () {
'use strict';

angular.module('Data')
.component('itemsList', {
  templateUrl: 'src/templates/items.component.html',
  bindings: {
    items: '<'
  }
});

})();
