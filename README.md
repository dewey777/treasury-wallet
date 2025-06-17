
```
[1단계] 유저 인터페이스
  - 웹사이트 접속 → wallet 연결 → 전송 버튼 클릭
  - 전송할 주소, 금액 입력 → Phantom 지갑 서명 요청 → 서명 및 승인

[2단계] 클라이언트 코드 (Anchor/JS)
  - transferSol(amount) 함수 실행
  - .accounts()로 필요한 계정들 전달
  - .signers()에 서명자 추가
  - .rpc()로 Solana 체인에 트랜잭션 전송

[3단계] Anchor 프로그램 실행
  - Context 검증 (계정 유효성 체크)
  - transfer_sol 함수 본문 실행:
      * from의 lamports 감소
      * to의 lamports 증가
      * 로그 출력

[4단계] Solana 네트워크
  - 트랜잭션 처리 및 블록에 포함
  - 상태 변화 영구 기록 (계정의 lamports 상태 변경됨)
  - 클라이언트에 트랜잭션 Signature 반환

[5단계] 클라이언트 후처리
  - 트랜잭션 결과 UI 표시
  - 최신 잔액 조회 / 로딩 표시
  - 사용자 알림 or Explorer 링크 제공


```