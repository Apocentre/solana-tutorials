/* eslint no-async-promise-executor: 0 */
import { Connection, clusterApiUrl } from '@solana/web3.js';
import Wallet from '@project-serum/sol-wallet-adapter'

const getProvider = () => {
  if ("solana" in window) {
    const provider = (window as any).solana;
    if (provider.isPhantom) {
      return provider;
    }
  }
  window.open("https://phantom.app/", "_blank");
}


export const init = () => new Promise((resolve, reject) => {
  const provider = getProvider()
  provider.connect({ onlyIfTrusted: true });
  provider
    .on("connect", () => {
      console.log("connected!")
      resolve(provider)
    })
})

