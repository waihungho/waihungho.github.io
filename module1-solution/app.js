(function () {
'use strict';

  angular.module('LunchCheck', [])

  .controller('LunchCheckController', LunchCheckController);

  LunchCheckController.$inject=['$scope'];

  function LunchCheckController($scope){
    $scope.returnmsg="";
    $scope.inputmsg ="";
    $scope.tryIt = function (){
      var msg = $scope.inputmsg;
      if (msg.length === 0 || !msg.trim()) {
        $scope.returnmsg ="Please enter data first";
      } else {
        if ( msg.split(",").length > 3 ) {
          $scope.returnmsg ="Too much!";
        } else {
          $scope.returnmsg ="Enjoy!";
        }
      }


    };
  }

})();
